use crate::{
    config::{IndexerConfig, MutableConfig},
    manifest::Module,
    ExecutionRequest, ExecutionResponse, Executor, IndexerResult, Manifest,
    NativeIndexExecutor, SchemaManager, WasmIndexExecutor,
};
use async_std::{fs::File, io::ReadExt, sync::Arc};
use fuel_gql_client::client::{
    FuelClient, PageDirection, PaginatedResult, PaginationRequest,
};
use fuel_indexer_database::{queries, IndexerConnection, IndexerConnectionPool};
use fuel_indexer_database_types::IndexAssetType;
use fuel_indexer_schema::{Address, BlockData, Bytes32};
use futures::stream::{futures_unordered::FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::future::Future;
use std::marker::{Send, Sync};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
    time::{sleep, Duration},
};

use tracing::{debug, error, info, warn};

const RETRY_LIMIT: usize = 5;

pub struct IndexerService {
    config: IndexerConfig,
    fuel_node_addr: SocketAddr,
    manager: SchemaManager,
    database_url: String,
    handles: HashMap<String, JoinHandle<()>>,
    killers: HashMap<String, Arc<AtomicBool>>,
    execution_request_rx: Receiver<ExecutionRequest>,
}

impl IndexerService {
    pub async fn new(
        config: IndexerConfig,
        execution_request_rx: Receiver<ExecutionRequest>,
    ) -> IndexerResult<IndexerService> {
        let database_url = config.database.to_string().clone();

        let manager = SchemaManager::new(&database_url).await?;

        let fuel_node_addr = config
            .fuel_node
            .clone()
            .derive_socket_addr()
            .expect("Could not parse Fuel node addr for IndexerService.");

        Ok(IndexerService {
            config,
            fuel_node_addr,
            manager,
            database_url,
            handles: HashMap::default(),
            killers: HashMap::default(),
            execution_request_rx,
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
                    .expect("Manifest should include GraphQL schema.");

                let schema_bytes = schema.as_bytes().to_vec();

                self.manager.new_schema(&namespace, &schema).await?;

                let (kill_switch, handle, wasm_bytes) = self
                    .spawn_executor_from_manifest(
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
                self.handles.insert(namespace.clone(), handle);
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

                    let (kill_switch, handle) = self
                        .spawn_executor_from_index_asset_registry(
                            &manifest,
                            run_once,
                            assets.wasm.bytes,
                        )
                        .await?;

                    info!("Registered indexer {}", manifest.uid());
                    self.handles.insert(manifest.namespace.clone(), handle);
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

    async fn spawn_executor_from_manifest(
        &self,
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

                let executor = WasmIndexExecutor::new(
                    database_url,
                    manifest.to_owned(),
                    bytes.clone(),
                )
                .await?;
                let kill_switch = Arc::new(AtomicBool::new(run_once));
                let handle = tokio::spawn(self.make_task(
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
                let handle = tokio::spawn(self.make_task(
                    kill_switch.clone(),
                    executor,
                    start_block,
                ));

                Ok((kill_switch, handle, None))
            }
        }
    }

    async fn spawn_executor_from_index_asset_registry(
        &self,
        manifest: &Manifest,
        run_once: bool,
        wasm_bytes: Vec<u8>,
    ) -> IndexerResult<(Arc<AtomicBool>, JoinHandle<()>)> {
        let start_block = manifest.start_block;

        match manifest.module {
            Module::Wasm(ref _module) => {
                let executor = WasmIndexExecutor::new(
                    self.database_url.clone(),
                    manifest.to_owned(),
                    wasm_bytes,
                )
                .await?;
                let kill_switch = Arc::new(AtomicBool::new(run_once));
                let handle = tokio::spawn(self.make_task(
                    kill_switch.clone(),
                    executor,
                    start_block,
                ));

                Ok((kill_switch, handle))
            }
            Module::Native(ref path) => {
                let path = path.clone();
                let executor = NativeIndexExecutor::new(
                    &self.database_url,
                    manifest.to_owned(),
                    path,
                )
                .await?;
                let kill_switch = Arc::new(AtomicBool::new(run_once));
                let handle = tokio::spawn(self.make_task(
                    kill_switch.clone(),
                    executor,
                    start_block,
                ));

                Ok((kill_switch, handle))
            }
        }
    }

    pub fn stop_indexer(&mut self, executor_name: &str) {
        if let Some(killer) = self.killers.remove(executor_name) {
            killer.store(true, Ordering::SeqCst);
        } else {
            warn!("Stop Indexer: No indexer with the name {executor_name}");
        }
    }

    fn make_task<T: 'static + Executor + Send + Sync>(
        &self,
        kill_switch: Arc<AtomicBool>,
        mut executor: T,
        start_block: Option<u64>,
    ) -> impl Future<Output = ()> {
        let mut next_cursor = None;
        let mut next_block = start_block.unwrap_or(1);
        let client = FuelClient::from(self.fuel_node_addr);

        async move {
            let mut retry_count = 0;

            loop {
                debug!("Fetching paginated results from {:?}", next_cursor);
                // TODO: can we have a "start at height" option?
                let PaginatedResult { cursor, results } = client
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
                    //       we'll need to watch contract creation events here in
                    //       case an indexer would be interested in processing it.
                    let mut transactions = Vec::new();
                    for trans in block.transactions {
                        match client.receipts(&trans.id.to_string()).await {
                            Ok(r) => {
                                transactions.push(r);
                            }
                            Err(e) => {
                                error!("Client communication error {:?}", e);
                            }
                        }
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
                        error!("Indexer failed after retries, giving up!");
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

    pub async fn run(&mut self) {
        let database_url = self.config.database.clone().to_string();
        let pool = IndexerConnectionPool::connect(&database_url)
            .await
            .expect("Execution task could not establish connection to database pool");

        while let Some(request) = self.execution_request_rx.recv().await {
            let mut conn = pool
                .acquire()
                .await
                .expect("Execution task could not acquire connection from pool");
            let assets = queries::latest_assets_for_index(&mut conn, &request.index_id)
                .await
                .expect("Could not get latest assets for index");

            let manifest: Manifest = serde_yaml::from_slice(&assets.manifest.bytes)
                .expect("Could not read manifest in registry.");

            let (kill_switch, handle) = self
                .spawn_executor_from_index_asset_registry(
                    &manifest,
                    false,
                    assets.wasm.bytes,
                )
                .await
                .expect("msbalhg");

            self.handles.insert(manifest.namespace.clone(), handle);
            self.killers.insert(manifest.namespace, kill_switch);

            // let _ = self
            //     .execution_response_tx
            //     .send(ExecutionResponse { executed: true })
            //     .await;
            // tokio::spawn(self.execute_new_assets(conn, request));
        }

        // let mut futs = FuturesUnordered::from_iter(self.handles.into_values());
        // while let Some(fut) = futs.next().await {
        //     debug!("Retired a future {fut:?}");
        // }
    }
}
