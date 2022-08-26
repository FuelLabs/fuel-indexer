use crate::{
    config::{AdjustableConfig, IndexerConfig},
    handler::Handle,
    Executor, IndexerResult, Manifest, NativeIndexExecutor, ReceiptEvent, SchemaManager,
    WasmIndexExecutor,
};
use async_std::sync::Arc;
use fuel_gql_client::client::{FuelClient, PageDirection, PaginatedResult, PaginationRequest};
use fuel_tx::Receipt;
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

pub struct IndexerService {
    fuel_node_addr: SocketAddr,
    manager: SchemaManager,
    database_url: String,
    handles: HashMap<String, JoinHandle<()>>,
    killers: HashMap<String, Arc<AtomicBool>>,
}

impl IndexerService {
    pub async fn new(config: IndexerConfig) -> IndexerResult<IndexerService> {
        let IndexerConfig {
            fuel_node,
            database,
            ..
        } = config;
        let manager = SchemaManager::new(&database.to_string()).await?;

        let fuel_node_addr = fuel_node
            .derive_socket_addr()
            .expect("Could not parse Fuel node addr for IndexerService.");

        Ok(IndexerService {
            fuel_node_addr,
            manager,
            database_url: database.to_string(),
            handles: HashMap::default(),
            killers: HashMap::default(),
        })
    }

    pub async fn add_native_indexer(
        &mut self,
        manifest: Manifest,
        run_once: bool,
        handles: Vec<Handle>,
    ) -> IndexerResult<()> {
        let name = manifest.namespace.clone();
        let start_block = manifest.start_block;

        let schema = manifest.load_schema_from_file().unwrap();
        self.manager.new_schema(&name, &schema).await?;
        let executor =
            NativeIndexExecutor::new(&self.database_url.clone(), manifest, handles).await?;

        let kill_switch = Arc::new(AtomicBool::new(run_once));
        let handle = tokio::spawn(self.make_task(
            ReceiptEvent::Other,
            kill_switch.clone(),
            executor,
            start_block,
        ));

        info!("Registered indexer {}", name);
        self.handles.insert(name.clone(), handle);
        self.killers.insert(name, kill_switch);

        Ok(())
    }

    pub async fn add_wasm_indexer(
        &mut self,
        manifest: Manifest,
        run_once: bool,
    ) -> IndexerResult<()> {
        let name = manifest.namespace.clone();
        let start_block = manifest.start_block;

        let schema = manifest.load_schema_from_file().unwrap();
        let wasm_bytes = manifest.load_wasm_from_file().unwrap();

        self.manager.new_schema(&name, &schema).await?;
        let executor =
            WasmIndexExecutor::new(self.database_url.clone(), manifest, wasm_bytes).await?;

        let kill_switch = Arc::new(AtomicBool::new(run_once));
        let handle = tokio::spawn(self.make_task(
            ReceiptEvent::Other,
            kill_switch.clone(),
            executor,
            start_block,
        ));

        info!("Registered indexer {}", name);
        self.handles.insert(name.clone(), handle);
        self.killers.insert(name, kill_switch);

        Ok(())
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
        _event_name: ReceiptEvent,
        kill_switch: Arc<AtomicBool>,
        executor: T,
        start_block: Option<u64>,
    ) -> impl Future<Output = ()> {
        let mut next_cursor = None;
        let mut next_block = start_block.unwrap_or(1);
        let client = FuelClient::from(self.fuel_node_addr);
        let executor = Arc::new(executor);

        async move {
            loop {
                debug!("Fetching paginated results from {:?}", next_cursor);
                // TODO: can we have a "start at height" option?
                let PaginatedResult { cursor, results } = client
                    .blocks(PaginationRequest {
                        cursor: next_cursor,
                        results: 10,
                        direction: PageDirection::Forward,
                    })
                    .await
                    .unwrap();

                debug!("Processing {} results", results.len());
                let exec = executor.clone();

                let mut receipts = Vec::new();
                for block in results.into_iter().rev() {
                    if block.height.0 != next_block {
                        continue;
                    }
                    next_block = block.height.0 + 1;
                    for trans in block.transactions {
                        match client.receipts(&trans.id.to_string()).await {
                            Ok(r) => {
                                receipts.extend(r);
                            }
                            Err(e) => {
                                error!("Client communication error {:?}", e);
                            }
                        }
                    }
                }

                let result = tokio::task::spawn(async move {
                    for receipt in receipts {
                        let receipt_cp = receipt.clone();
                        match receipt {
                            Receipt::LogData {
                                // TODO: use data field for now, the rest will be useful later
                                data,
                                ..
                            } => {
                                // TODO: should wrap this in a db transaction.
                                if let Err(e) = exec
                                    .trigger_event(
                                        ReceiptEvent::LogData,
                                        vec![data],
                                        Some(receipt_cp),
                                    )
                                    .await
                                {
                                    error!("Event processing failed {:?}", e);
                                }
                            }
                            Receipt::ReturnData {
                                // TODO: use data field for now, the rest will be useful later
                                data,
                                ..
                            } => {
                                // TODO: should wrap this in a db transaction.
                                if let Err(e) = exec
                                    .trigger_event(
                                        ReceiptEvent::ReturnData,
                                        vec![data],
                                        Some(receipt_cp),
                                    )
                                    .await
                                {
                                    error!("Event processing failed {:?}", e);
                                }
                            }
                            o => warn!("Unhandled receipt type: {:?}", o),
                        }
                    }
                })
                .await;

                if let Err(e) = result {
                    error!("Indexer executor failed {e:?}");
                }

                next_cursor = cursor;
                if next_cursor.is_none() {
                    info!("No next page, sleeping");
                    sleep(Duration::from_secs(5)).await;
                };

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
            info!("Retired a future {fut:?}");
        }
    }
}
