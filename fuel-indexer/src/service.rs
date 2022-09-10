use crate::{
    config::{IndexerConfig, MutableConfig},
    manifest::Module,
    Executor, IndexerResult, Manifest, NativeIndexExecutor, SchemaManager, WasmIndexExecutor,
};
use async_std::{fs::File, io::ReadExt, sync::Arc};
use fuel_gql_client::client::{FuelClient, PageDirection, PaginatedResult, PaginationRequest};
use fuel_indexer_schema::{
    db::{IndexerConnection, IndexerConnectionPool},
    BlockData,
};
use futures::stream::{futures_unordered::FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::future::Future;
use std::marker::{Send, Sync};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{
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
}

impl IndexerService {
    pub async fn new(config: IndexerConfig) -> IndexerResult<IndexerService> {
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
        })
    }

    // TODO: run_once should be configurable (on a per-index basis  - e.g., in the manifest)
    pub async fn register_indices(
        &mut self,
        manifest: Option<Manifest>,
        run_once: bool,
    ) -> IndexerResult<()> {
        let database_url = self.database_url.clone();

        let conn = IndexerConnectionPool::connect(&self.config.database.to_string()).await?;
        let pool = conn.acquire().await?;

        match manifest {
            Some(manifest) => {
                let namespace = manifest.namespace.clone();
                let identifier = manifest.identifier.clone();

                let schema = manifest
                    .graphql_schema()
                    .expect("Manifest should include GraphQL schema");
                self.manager.new_schema(&namespace, &schema).await?;

                let (kill_switch, handle, wasm_bytes) = self
                    .spawn_executor_from_manifest(&manifest, run_once, database_url.clone())
                    .await?;

                match pool {
                    IndexerConnection::Postgres(mut c) => {
                        fuel_indexer_postgres::register_index_assets(
                            &mut c,
                            &namespace,
                            &identifier,
                            Some(wasm_bytes.unwrap()),
                            Some(manifest.to_bytes()),
                            Some(schema.as_bytes().to_vec()),
                        )
                        .await
                    }
                    IndexerConnection::Sqlite(mut c) => {
                        fuel_indexer_sqlite::register_index_assets(
                            &mut c,
                            &namespace,
                            &identifier,
                            None,
                            Some(manifest.to_bytes()),
                            Some(schema.as_bytes().to_vec()),
                        )
                        .await
                    }
                }
                .expect("Failed");

                // TODO: indices hsould be indexed by UID
                info!("Registered indexer {}", identifier);
                self.handles.insert(namespace.clone(), handle);
                self.killers.insert(namespace, kill_switch);
            }
            None => {
                let registered_assets = match pool {
                    IndexerConnection::Postgres(mut c) => {
                        fuel_indexer_postgres::get_all_registered_assets(&mut c).await
                    }
                    IndexerConnection::Sqlite(mut c) => {
                        fuel_indexer_sqlite::get_all_registered_assets(&mut c).await
                    }
                }
                .expect("Failed");

                for asset in registered_assets {
                    let manifest: Manifest = serde_yaml::from_slice(&asset.manifest).expect("Bar");

                    let (kill_switch, handle) = self
                        .spwan_executor_from_asset_registry(&manifest, run_once, asset.wasm)
                        .await?;

                    // TODO: indices hsould be indexed by UID
                    info!("Registered indexer {}", manifest.uid());
                    self.handles.insert(manifest.namespace.clone(), handle);
                    self.killers.insert(manifest.namespace, kill_switch);
                }
            }
        }

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

                let executor =
                    WasmIndexExecutor::new(database_url, manifest.to_owned(), bytes.clone())
                        .await?;
                let kill_switch = Arc::new(AtomicBool::new(run_once));
                let handle =
                    tokio::spawn(self.make_task(kill_switch.clone(), executor, start_block));

                Ok((kill_switch, handle, Some(bytes)))
            }
            Module::Native(ref path) => {
                let path = path.clone();
                let executor =
                    NativeIndexExecutor::new(&database_url, manifest.to_owned(), path).await?;
                let kill_switch = Arc::new(AtomicBool::new(run_once));
                let handle =
                    tokio::spawn(self.make_task(kill_switch.clone(), executor, start_block));

                Ok((kill_switch, handle, None))
            }
        }
    }

    async fn spwan_executor_from_asset_registry(
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
                let handle =
                    tokio::spawn(self.make_task(kill_switch.clone(), executor, start_block));

                Ok((kill_switch, handle))
            }
            Module::Native(ref path) => {
                let path = path.clone();
                let executor =
                    NativeIndexExecutor::new(&self.database_url, manifest.to_owned(), path).await?;
                let kill_switch = Arc::new(AtomicBool::new(run_once));
                let handle =
                    tokio::spawn(self.make_task(kill_switch.clone(), executor, start_block));

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

    pub async fn run(self) {
        let IndexerService { handles, .. } = self;
        let mut futs = FuturesUnordered::from_iter(handles.into_values());
        while let Some(fut) = futs.next().await {
            debug!("Retired a future {fut:?}");
        }
    }
}
