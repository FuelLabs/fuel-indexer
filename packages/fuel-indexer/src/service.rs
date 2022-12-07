use crate::{
    Executor, IndexerConfig, IndexerResult, Manifest, Module, NativeIndexExecutor,
    WasmIndexExecutor,
};
use async_std::{fs::File, io::ReadExt};
use chrono::{TimeZone, Utc};
use fuel_gql_client::client::{
    types::{TransactionResponse, TransactionStatus as GqlTransactionStatus},
    FuelClient, PageDirection, PaginatedResult, PaginationRequest,
};
use fuel_indexer_database::{queries, types::IndexAssetType, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::FuelNodeConfig,
    defaults::{
        DATABASE_CONNECTION_RETRY_ATTEMPTS, DELAY_FOR_EMPTY_PAGE, DELAY_FOR_SERVICE_ERR,
    },
    utils::ServiceRequest,
};
use fuel_indexer_schema::db::manager::SchemaManager;
use fuel_indexer_types::{
    abi::{BlockData, TransactionData},
    tx::{TransactionStatus, TxId},
    Bytes32,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::marker::{Send, Sync};
use std::str::FromStr;
use tokio::{
    sync::mpsc::Receiver,
    task::JoinHandle,
    time::{sleep, Duration},
};
use tracing::{debug, error, info, warn};

async fn create_executor(
    fuel_node: FuelNodeConfig,
    db_url: String,
    manifest: &Manifest,
    module_bytes: Option<Vec<u8>>,
) -> IndexerResult<(JoinHandle<()>, Option<Vec<u8>>)> {
    // If the task creation is via index asset registry
    if module_bytes.is_none() {
        match &manifest.module {
            Module::Wasm(ref module) => {
                let mut bytes = Vec::<u8>::new();
                let mut file = File::open(module).await?;
                file.read_to_end(&mut bytes).await?;

                let executor =
                    WasmIndexExecutor::new(db_url, manifest.to_owned(), bytes.clone())
                        .await?;
                let handle = tokio::spawn(run_executor(
                    &fuel_node.to_string(),
                    executor,
                    manifest.start_block,
                ));

                Ok((handle, Some(bytes)))
            }
            Module::Native(ref _path) => {
                let executor =
                    NativeIndexExecutor::new(&db_url, manifest.to_owned()).await?;
                let handle = tokio::spawn(run_executor(
                    &fuel_node.to_string(),
                    executor,
                    manifest.start_block,
                ));

                Ok((handle, None))
            }
        }
    // If the task creation is via the bootstrap manifest
    } else {
        let executor = WasmIndexExecutor::new(
            db_url,
            manifest.to_owned(),
            module_bytes.clone().unwrap(),
        )
        .await?;
        let handle = tokio::spawn(run_executor(
            &fuel_node.to_string(),
            executor,
            manifest.start_block,
        ));

        Ok((handle, module_bytes))
    }
}

fn run_executor<T: 'static + Executor + Send + Sync>(
    fuel_node_addr: &str,
    mut executor: T,
    start_block: Option<u64>,
) -> impl Future<Output = ()> {
    let start_block_value = start_block.unwrap_or(1);
    // cursor will return result from block + 1, so negating with 1 to start from `start_block - 1`
    let mut next_cursor = if start_block_value > 1 {
        let decremented = start_block_value - 1;
        Some(decremented.to_string())
    } else {
        None
    };

    info!("Subscribing to Fuel node at {}", fuel_node_addr);

    let client = FuelClient::from_str(fuel_node_addr).unwrap_or_else(|e| {
        panic!(
            "Unable to connect to Fuel node at '{}': {}",
            fuel_node_addr, e
        )
    });

    async move {
        let mut retry_count = 0;

        loop {
            debug!("Fetching paginated results from {:?}", next_cursor);

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
                // NOTE: for now assuming we have a single contract instance,
                // we'll need to watch contract creation events here in
                // case an indexer would be interested in processing it.
                let mut transactions = Vec::new();

                for trans in block.transactions {
                    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/288
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
                                    } => TransactionStatus::Success {
                                        block_id,
                                        time: Utc
                                            .timestamp_opt(time.to_unix(), 0)
                                            .single()
                                            .expect(
                                                "Failed to parse transaction timestamp",
                                            ),
                                    },
                                    GqlTransactionStatus::Failure {
                                        block_id,
                                        time,
                                        reason,
                                        ..
                                    } => TransactionStatus::Failure {
                                        block_id,
                                        time: Utc
                                            .timestamp_opt(time.to_unix(), 0)
                                            .single()
                                            .expect(
                                                "Failed to parse transaction timestamp",
                                            ),
                                        reason,
                                    },
                                    GqlTransactionStatus::Submitted { submitted_at } => {
                                        TransactionStatus::Submitted {
                                            submitted_at: Utc
                                                .timestamp_opt(submitted_at.to_unix(), 0)
                                                .single()
                                                .expect(
                                                    "Failed to parse transaction timestamp"
                                                ),
                                        }
                                    }
                                    GqlTransactionStatus::SqueezedOut { reason } => {
                                        TransactionStatus::SqueezedOut { reason }
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
                    height: block.header.height.0,
                    id: Bytes32::from(block.id),
                    time: block.header.time.0.to_unix(),
                    transactions,
                };

                block_info.push(block);
            }

            let result = executor.handle_events(block_info).await;

            if let Err(e) = result {
                error!("Indexer executor failed {e:?}, retrying.");
                sleep(Duration::from_secs(DELAY_FOR_SERVICE_ERR)).await;
                retry_count += 1;
                if retry_count < DATABASE_CONNECTION_RETRY_ATTEMPTS {
                    continue;
                } else {
                    error!("Indexer failed after retries, giving up.");
                    break;
                }
            }

            if cursor.is_none() {
                info!("No new blocks to process, sleeping.");
                sleep(Duration::from_secs(DELAY_FOR_EMPTY_PAGE)).await;
            } else {
                next_cursor = cursor;
            }

            retry_count = 0;
        }
    }
}

pub struct IndexerService {
    config: IndexerConfig,
    rx: Option<Receiver<ServiceRequest>>,
    pool: IndexerConnectionPool,
    manager: SchemaManager,
    database_url: String,
    handles: RefCell<HashMap<String, JoinHandle<()>>>,
}

impl IndexerService {
    pub async fn new(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        rx: Option<Receiver<ServiceRequest>>,
    ) -> IndexerResult<IndexerService> {
        let database_url = config.database.to_string();

        let manager = SchemaManager::new(pool.clone());

        Ok(IndexerService {
            config,
            rx,
            pool,
            manager,
            database_url,
            handles: RefCell::new(HashMap::default()),
        })
    }

    pub async fn register_index_from_manifest(
        &mut self,
        manifest: Manifest,
    ) -> IndexerResult<()> {
        let database_url = self.database_url.clone();

        let mut conn = self.pool.acquire().await?;

        let _ = queries::start_transaction(&mut conn).await?;

        let index =
            queries::register_index(&mut conn, &manifest.namespace, &manifest.identifier)
                .await?;

        let schema = manifest
            .graphql_schema()
            .expect("Failed to read GraphQL schema file in manifest.");

        let schema_bytes = schema.as_bytes().to_vec();

        self.manager
            .new_schema(&manifest.namespace, &schema)
            .await?;

        let (handle, module_bytes) = create_executor(
            self.config.fuel_node.clone(),
            database_url.clone(),
            &manifest,
            None,
        )
        .await?;

        let mut items = vec![
            (IndexAssetType::Wasm, module_bytes.unwrap()),
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
                    &manifest.namespace,
                    &manifest.identifier,
                    bytes,
                    asset_type,
                )
                .await?;
            }
        }

        let _ = queries::commit_transaction(&mut conn).await?;

        info!("Registered Index({})", &manifest.uid());
        self.handles.borrow_mut().insert(manifest.uid(), handle);

        Ok(())
    }

    pub async fn register_indices_from_registry(&mut self) -> IndexerResult<()> {
        let mut conn = self.pool.acquire().await?;

        let _ = queries::start_transaction(&mut conn)
            .await
            .expect("Could not start database transaction");

        let indices = queries::registered_indices(&mut conn).await?;

        println!(">>> INDICIES : {:?}", indices);
        for index in indices {
            let assets = queries::latest_assets_for_index(&mut conn, &index.id).await?;
            let manifest = Manifest::from_slice(&assets.manifest.bytes)?;

            let handle = create_executor(
                self.config.fuel_node.clone(),
                self.config.database.to_string(),
                &manifest,
                Some(assets.wasm.bytes),
            )
            .await?
            .0;

            info!("Registered indexer {}", manifest.uid());
            self.handles.borrow_mut().insert(manifest.uid(), handle);
        }

        let _ = match queries::commit_transaction(&mut conn).await {
            Ok(v) => v,
            Err(_e) => queries::revert_transaction(&mut conn).await?,
        };

        Ok(())
    }

    pub async fn run(self) {
        let IndexerService {
            handles, mut rx, ..
        } = self;

        if let Some(ref mut rx) = rx {
            while let Some(service_request) = rx.recv().await {
                match service_request {
                    ServiceRequest::AssetReload(request) => {
                        let mut conn = self
                            .pool
                            .acquire()
                            .await
                            .expect("Failed to acquire connection from pool");

                        let index_id = queries::index_id_for(
                            &mut conn,
                            &request.namespace,
                            &request.identifier,
                        )
                        .await
                        .unwrap_or_else(|_| {
                            panic!(
                                "Failed to get id for {}.{}",
                                &request.namespace, &request.identifier
                            )
                        });

                        let assets =
                            queries::latest_assets_for_index(&mut conn, &index_id)
                                .await
                                .expect("Could not get latest assets for index");

                        let manifest: Manifest =
                            serde_yaml::from_slice(&assets.manifest.bytes)
                                .expect("Failed to deserialize manifest");

                        let handle = create_executor(
                            self.config.fuel_node.clone(),
                            self.config.database.to_string(),
                            &manifest,
                            Some(assets.wasm.bytes),
                        )
                        .await
                        .expect("Failed to spawn executor from index asset registry")
                        .0;

                        handles.borrow_mut().insert(manifest.uid(), handle);
                    }

                    ServiceRequest::IndexStop(request) => {
                        let uid = format!("{}.{}", request.namespace, request.identifier);
                        if let Some(handle) = handles.borrow_mut().remove(&uid) {
                            handle.abort();
                            info!("Stopped index for {}", &uid);
                        } else {
                            warn!("Stop Indexer: No index with the name {}", &uid);
                        }
                    }
                }
            }
        }
    }
}
