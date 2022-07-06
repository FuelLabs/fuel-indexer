use crate::{
    handler::Handle, Executor, IndexerResult, Manifest, NativeIndexExecutor, ReceiptEvent,
    SchemaManager, WasmIndexExecutor,
};
use anyhow::Result;
use async_std::{fs::File, io::ReadExt, sync::Arc};
use fuel_gql_client::client::{FuelClient, PageDirection, PaginatedResult, PaginationRequest};
use fuel_tx::Receipt;
use fuels_core::abi_encoder::ABIEncoder;
use fuels_core::{Token, Tokenizable};
use futures::stream::{futures_unordered::FuturesUnordered, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::future::Future;
use std::marker::{Send, Sync};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};

use tracing::{debug, error, info, warn};

#[derive(Clone, Deserialize)]
pub struct IndexerConfig {
    pub fuel_node_addr: SocketAddr,
    pub database_url: String,
    pub listen_endpoint: SocketAddr,
}

impl IndexerConfig {
    pub async fn from_file(path: &Path) -> Result<Self> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let config: IndexerConfig = serde_yaml::from_str(&contents)?;

        Ok(config)
    }
}

pub struct IndexerService {
    fuel_node_addr: SocketAddr,
    manager: SchemaManager,
    database_url: String,
    handles: HashMap<String, JoinHandle<()>>,
    killers: HashMap<String, Arc<AtomicBool>>,
}

impl IndexerService {
    pub fn new(config: IndexerConfig) -> IndexerResult<IndexerService> {
        let IndexerConfig {
            fuel_node_addr,
            database_url,
            ..
        } = config;
        let manager = SchemaManager::new(&database_url)?;

        Ok(IndexerService {
            fuel_node_addr,
            manager,
            database_url,
            handles: HashMap::default(),
            killers: HashMap::default(),
        })
    }

    pub fn add_native_indexer(
        &mut self,
        manifest: Manifest,
        run_once: bool,
        handles: Vec<Handle>,
    ) -> IndexerResult<()> {
        let name = manifest.namespace.clone();
        let start_block = manifest.start_block;

        let schema = manifest.graphql_schema().unwrap();
        self.manager.new_schema(&name, &schema)?;
        let executor = NativeIndexExecutor::new(&self.database_url.clone(), manifest, handles)?;

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

    pub fn add_wasm_indexer(&mut self, manifest: Manifest, run_once: bool) -> IndexerResult<()> {
        let name = manifest.namespace.clone();
        let start_block = manifest.start_block;

        let schema = manifest.graphql_schema().unwrap();
        let wasm_bytes = manifest.wasm_module().unwrap();

        self.manager.new_schema(&name, &schema)?;
        let executor = WasmIndexExecutor::new(self.database_url.clone(), manifest, wasm_bytes)?;

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

                let result = tokio::task::spawn_blocking(move || {
                    for receipt in receipts {
                        let receipt_cp = receipt.clone();
                        match receipt {
                            Receipt::Log {
                                id,
                                ra,
                                rb,
                                rc,
                                rd,
                                pc,
                                is,
                            } => {
                                // TODO: might be nice to have Receipt type impl Tokenizable.
                                let token = Token::Struct(vec![
                                    id.into_token(),
                                    ra.into_token(),
                                    rb.into_token(),
                                    rc.into_token(),
                                    rd.into_token(),
                                    pc.into_token(),
                                    is.into_token(),
                                ]);

                                let args = ABIEncoder::new()
                                    .encode(&[token.clone()])
                                    .expect("Bad Encoding!");
                                // TODO: should wrap this in a db transaction.
                                if let Err(e) = exec.trigger_event(
                                    ReceiptEvent::Log,
                                    vec![args],
                                    Some(receipt_cp),
                                ) {
                                    error!("Event processing failed {:?}", e);
                                }
                            }
                            Receipt::LogData {
                                id,
                                ra,
                                rb,
                                ptr,
                                len,
                                digest,
                                data,
                                pc,
                                is,
                            } => {
                                // TODO: might be nice to have Receipt type impl Tokenizable.
                                let token = Token::Struct(vec![
                                    id.into_token(),
                                    ra.into_token(),
                                    rb.into_token(),
                                    ptr.into_token(),
                                    len.into_token(),
                                    digest.into_token(),
                                    data.into_token(),
                                    pc.into_token(),
                                    is.into_token(),
                                ]);

                                let args = ABIEncoder::new()
                                    .encode(&[token.clone()])
                                    .expect("Bad Encoding!");
                                // TODO: should wrap this in a db transaction.
                                if let Err(e) = exec.trigger_event(
                                    ReceiptEvent::LogData,
                                    vec![args],
                                    Some(receipt_cp),
                                ) {
                                    error!("Event processing failed {:?}", e);
                                }
                            }
                            Receipt::ReturnData {
                                id,
                                ptr,
                                len,
                                digest,
                                data,
                                pc,
                                is,
                            } => {
                                // TODO: might be nice to have Receipt type impl Tokenizable.
                                let token = Token::Struct(vec![
                                    id.into_token(),
                                    ptr.into_token(),
                                    len.into_token(),
                                    digest.into_token(),
                                    data.into_token(),
                                    pc.into_token(),
                                    is.into_token(),
                                ]);

                                let args = ABIEncoder::new()
                                    .encode(&[token.clone()])
                                    .expect("Bad Encoding!");
                                // TODO: should wrap this in a db transaction.
                                if let Err(e) = exec.trigger_event(
                                    ReceiptEvent::ReturnData,
                                    vec![args],
                                    Some(receipt_cp),
                                ) {
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
