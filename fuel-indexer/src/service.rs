use crate::{
    config::{FuelNodeConfig, IndexerConfig},
    Executor, IndexerResult, Manifest, Module, NativeIndexExecutor, SchemaManager,
    WasmIndexExecutor,
};
use async_std::{fs::File, io::ReadExt, sync::Arc};
use fuel_gql_client::client::{
    types::{TransactionResponse, TransactionStatus as GqlTransactionStatus},
    FuelClient, PageDirection, PaginatedResult, PaginationRequest,
};
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_database_types::IndexAssetType;
use fuel_indexer_lib::utils::AssetReloadRequest;
use fuel_indexer_schema::types::{
    Address, BlockData, Bytes32, TransactionData, TransactionStatus,
};
use fuel_tx::TxId;
use futures::stream::{futures_unordered::FuturesUnordered, StreamExt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::marker::{Send, Sync};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{
    sync::mpsc::Receiver,
    task::JoinHandle,
    time::{sleep, Duration},
};

use tracing::{debug, error, info, warn};

const RETRY_LIMIT: usize = 5;

async fn spawn_executor_from_manifest(
    fuel_node: FuelNodeConfig,
    manifest: &Manifest,
    run_once: bool,
    database_url: String,
) -> IndexerResult<(Arc<AtomicBool>, JoinHandle<()>, Option<Vec<u8>>)> {
    let start_block = manifest.start_block;

    match manifest.module {
        Module::Wasm(ref module) => {
            let mut bytes = Vec::<u8>::new();
            let mut file = File::open(module).await?;
            file.read_to_end(&mut bytes).await?;

            let executor =
                WasmIndexExecutor::new(database_url, manifest.to_owned(), bytes.clone())
                    .await?;
            let kill_switch = Arc::new(AtomicBool::new(run_once));
            let handle = tokio::spawn(make_task(
                fuel_node.into(),
                kill_switch.clone(),
                executor,
                start_block,
            ));

            Ok((kill_switch, handle, Some(bytes)))
        }
        Module::Native(ref path) => {
            let path = path.clone();
            let executor =
                NativeIndexExecutor::new(&database_url, manifest.to_owned(), path)
                    .await?;
            let kill_switch = Arc::new(AtomicBool::new(run_once));
            let handle = tokio::spawn(make_task(
                fuel_node.into(),
                kill_switch.clone(),
                executor,
                start_block,
            ));

            Ok((kill_switch, handle, None))
        }
    }
}

async fn spawn_executor_from_index_asset_registry(
    fuel_node: FuelNodeConfig,
    db_url: String,
    manifest: &Manifest,
    run_once: bool,
    wasm_bytes: Vec<u8>,
) -> IndexerResult<(Arc<AtomicBool>, JoinHandle<()>)> {
    let start_block = manifest.start_block;

    match manifest.module {
        Module::Wasm(ref _module) => {
            let executor =
                WasmIndexExecutor::new(db_url, manifest.to_owned(), wasm_bytes).await?;
            let kill_switch = Arc::new(AtomicBool::new(run_once));
            let handle = tokio::spawn(make_task(
                fuel_node.into(),
                kill_switch.clone(),
                executor,
                start_block,
            ));

            Ok((kill_switch, handle))
        }
        Module::Native(ref path) => {
            let path = path.clone();
            let executor =
                NativeIndexExecutor::new(&db_url, manifest.to_owned(), path).await?;
            let kill_switch = Arc::new(AtomicBool::new(run_once));
            let handle = tokio::spawn(make_task(
                fuel_node.into(),
                kill_switch.clone(),
                executor,
                start_block,
            ));

            Ok((kill_switch, handle))
        }
    }
}

fn make_task<T: 'static + Executor + Send + Sync>(
    fuel_node_addr: SocketAddr,
    kill_switch: Arc<AtomicBool>,
    mut executor: T,
    start_block: Option<u64>,
) -> impl Future<Output = ()> {
    let mut next_cursor = None;
    let mut next_block = start_block.unwrap_or(1);
    let client = FuelClient::from(fuel_node_addr);

    async move {
        let mut retry_count = 0;

        loop {
            debug!("Fetching paginated results from {:?}", next_cursor);
            // TODO: can we have a "start at height" option?
            let PaginatedResult {
                cursor, results, ..
            } = client
                .blocks(PaginationRequest {
                    cursor: next_cursor.clone(),
                    results: 10,
                    direction: PageDirection::Forward,
                })
                .await
                .expect("Failed to retrieve blocks");

            debug!("Processing {} results", results.len());

            let mut block_info = Vec::new();
            for block in results.into_iter().rev() {
                if block.height.0 != next_block {
                    continue;
                }
                next_block = block.height.0 + 1;

                // NOTE: for now assuming we have a single contract instance,
                // we'll need to watch contract creation events here in
                // case an indexer would be interested in processing it.
                let mut transactions = Vec::new();

                for trans in block.transactions {
                    // TODO: We should optimize this a bit by using client.transactions(), but need
                    // to make sure that the paginated response contains only the transactions
                    // in block.transactions (should be the exact same, just full transactions as opposed
                    // to TransactionIdFragments)
                    match client.transaction(&trans.id.to_string()).await {
                        Ok(result) => {
                            if let Some(TransactionResponse {
                                transaction,
                                status,
                            }) = result
                            {
                                let receipts = match client
                                    .receipts(&trans.id.to_string())
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        error!(
                                            "Client communication error fetching receipts: {:?}",
                                            e
                                        );
                                        vec![]
                                    }
                                };

                                // NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
                                let status = match status {
                                    GqlTransactionStatus::Success {
                                        block_id,
                                        time,
                                        ..
                                    } => TransactionStatus::Success { block_id, time },
                                    GqlTransactionStatus::Failure {
                                        block_id,
                                        time,
                                        reason,
                                        ..
                                    } => TransactionStatus::Failure {
                                        block_id,
                                        time,
                                        reason,
                                    },
                                    GqlTransactionStatus::Submitted { submitted_at } => {
                                        TransactionStatus::Submitted { submitted_at }
                                    }
                                };

                                let tx_data = TransactionData {
                                    receipts,
                                    status,
                                    transaction,
                                    id: TxId::from(trans.id),
                                };
                                transactions.push(tx_data);
                            }
                        }
                        Err(e) => {
                            error!(
                                "Client communication error fetching transactions: {:?}",
                                e
                            )
                        }
                    };
                }

                let block = BlockData {
                    height: block.height.0,
                    id: Bytes32::from(block.id),
                    time: block.time.timestamp(),
                    producer: Address::from(block.producer),
                    transactions,
                };

                block_info.push(block);
            }

            let result = executor.handle_events(block_info).await;

            if let Err(e) = result {
                error!("Indexer executor failed {e:?}, retrying.");
                sleep(Duration::from_secs(5)).await;
                retry_count += 1;
                if retry_count < RETRY_LIMIT {
                    continue;
                } else {
                    error!("Indexer failed after retries, giving up.");
                    break;
                }
            }

            next_cursor = cursor;
            if next_cursor.is_none() {
                info!("No next page, sleeping");
                sleep(Duration::from_secs(5)).await;
            };
            retry_count = 0;

            if kill_switch.load(Ordering::SeqCst) {
                break;
            }
        }
    }
}

pub struct IndexerService {
    config: IndexerConfig,
    rx: Option<Receiver<AssetReloadRequest>>,
    manager: SchemaManager,
    database_url: String,
    handles: RefCell<HashMap<String, JoinHandle<()>>>,
    killers: HashMap<String, Arc<AtomicBool>>,
}

impl IndexerService {
    pub async fn new(
        config: IndexerConfig,
        rx: Option<Receiver<AssetReloadRequest>>,
    ) -> IndexerResult<IndexerService> {
        let database_url = config.database.to_string().clone();

        let manager = SchemaManager::new(&database_url).await?;

        Ok(IndexerService {
            config,
            rx,
            manager,
            database_url,
            handles: RefCell::new(HashMap::default()),
            killers: HashMap::default(),
        })
    }

    // TODO: run_once should come from the index's manifest
    pub async fn register_indices(
        &mut self,
        manifest: Option<Manifest>,
        run_once: bool,
    ) -> IndexerResult<()> {
        let database_url = self.database_url.clone();

        let pool =
            IndexerConnectionPool::connect(&self.config.database.to_string()).await?;
        let mut conn = pool.acquire().await?;

        let _ = queries::start_transaction(&mut conn)
            .await
            .expect("Could not start database transaction");

        match manifest {
            Some(manifest) => {
                let namespace = manifest.namespace.clone();
                let identifier = manifest.identifier.clone();

                let index =
                    queries::register_index(&mut conn, &namespace, &identifier).await?;

                let schema = manifest
                    .graphql_schema()
                    .expect("Failed to read GraphQL schema file in manifest.");

                let schema_bytes = schema.as_bytes().to_vec();

                self.manager.new_schema(&namespace, &schema).await?;

                let (kill_switch, handle, wasm_bytes) = spawn_executor_from_manifest(
                    self.config.fuel_node.clone(),
                    &manifest,
                    run_once,
                    database_url.clone(),
                )
                .await?;

                let mut items = vec![
                    (IndexAssetType::Wasm, wasm_bytes.unwrap()),
                    (IndexAssetType::Manifest, manifest.to_bytes()),
                    (IndexAssetType::Schema, schema_bytes),
                ];

                while let Some((asset_type, bytes)) = items.pop() {
                    info!(
                        "Registering Asset({:?}) for Index({})",
                        asset_type,
                        index.uid()
                    );
                    {
                        queries::register_index_asset(
                            &mut conn,
                            &namespace,
                            &identifier,
                            bytes,
                            asset_type,
                        )
                        .await?;
                    }
                }

                info!("Registered indexer {}", identifier);
                self.handles.borrow_mut().insert(namespace.clone(), handle);
                self.killers.insert(namespace, kill_switch);
            }
            None => {
                let indices = queries::registered_indices(&mut conn).await?;
                for index in indices {
                    let assets =
                        queries::latest_assets_for_index(&mut conn, &index.id).await?;
                    let manifest: Manifest =
                        serde_yaml::from_slice(&assets.manifest.bytes)
                            .expect("Could not read manifest in registry.");

                    let (kill_switch, handle) = spawn_executor_from_index_asset_registry(
                        self.config.fuel_node.clone(),
                        self.config.database.to_string(),
                        &manifest,
                        run_once,
                        assets.wasm.bytes,
                    )
                    .await?;

                    info!("Registered indexer {}", manifest.uid());
                    self.handles
                        .borrow_mut()
                        .insert(manifest.namespace.clone(), handle);
                    self.killers.insert(manifest.namespace, kill_switch);
                }
            }
        }

        let _ = match queries::commit_transaction(&mut conn).await {
            Ok(v) => v,
            Err(_e) => queries::revert_transaction(&mut conn)
                .await
                .expect("Could not revert database transaction"),
        };

        Ok(())
    }

    pub fn stop_indexer(&mut self, executor_name: &str) {
        if let Some(killer) = self.killers.remove(executor_name) {
            killer.store(true, Ordering::SeqCst);
        } else {
            warn!("Stop Indexer: No indexer with the name {executor_name}");
        }
    }

    pub async fn run(self) {
        let IndexerService {
            handles,
            mut rx,
            mut killers,
            ..
        } = self;
        let mut futs = FuturesUnordered::from_iter(handles.take().into_values());

        while let Some(fut) = futs.next().await {
            debug!("Retired a future {fut:?}");
        }

        if let Some(ref mut rx) = rx {
            while let Some(request) = rx.recv().await {
                debug!("Retired a future {request:?}");

                let database_url = self.config.database.clone().to_string();
                let pool = IndexerConnectionPool::connect(&database_url).await.unwrap();

                let mut conn = pool.acquire().await.unwrap();

                let index_id = queries::index_id_for(
                    &mut conn,
                    &request.namespace,
                    &request.identifier,
                )
                .await
                .unwrap();

                let assets = queries::latest_assets_for_index(&mut conn, &index_id)
                    .await
                    .expect("Could not get latest assets for index");

                let manifest: Manifest =
                    serde_yaml::from_slice(&assets.manifest.bytes).unwrap();

                let (kill_switch, handle) = spawn_executor_from_index_asset_registry(
                    self.config.fuel_node.clone(),
                    self.config.database.to_string(),
                    &manifest,
                    false,
                    assets.wasm.bytes,
                )
                .await
                .unwrap();

                handles
                    .borrow_mut()
                    .insert(request.namespace.clone(), handle);
                killers.insert(request.namespace, kill_switch);
            }
        }
    }
}
