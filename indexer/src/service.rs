use crate::{
    handler::Handle, Executor, FuelNodeConfig, GraphQLConfig, IndexerArgs, IndexerResult, Manifest,
    NativeIndexExecutor, PostgresConfig, ReceiptEvent, SchemaManager, WasmIndexExecutor,
};
use anyhow::Result;
use async_std::{fs::File, io::ReadExt, sync::Arc};
use fuel_gql_client::client::{FuelClient, PageDirection, PaginatedResult, PaginationRequest};
use fuel_indexer_lib::{config::InjectEnvironment, defaults};
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

#[derive(Clone, Deserialize, Default, Debug)]
pub struct IndexerConfig {
    pub fuel_node: FuelNodeConfig,
    pub graphql_api: GraphQLConfig,
    pub postgres: PostgresConfig,
}

#[derive(Deserialize)]
pub struct TmpIndexerConfig {
    pub fuel_node: Option<FuelNodeConfig>,
    pub graphql_api: Option<GraphQLConfig>,
    pub postgres: Option<PostgresConfig>,
}

impl IndexerConfig {
    pub fn upgrade_optionals(&mut self, tmp: TmpIndexerConfig) {
        if let Some(cfg) = tmp.fuel_node {
            self.fuel_node = cfg;
        }

        if let Some(cfg) = tmp.postgres {
            self.postgres = cfg;
        }

        if let Some(cfg) = tmp.graphql_api {
            self.graphql_api = cfg;
        }
    }

    pub fn from_opts(args: IndexerArgs) -> IndexerConfig {
        IndexerConfig {
            fuel_node: FuelNodeConfig {
                host: args
                    .fuel_node_host
                    .unwrap_or_else(|| defaults::FUEL_NODE_HOST.into()),
                port: args
                    .fuel_node_port
                    .unwrap_or_else(|| defaults::FUEL_NODE_PORT.into()),
            },
            postgres: PostgresConfig {
                user: args
                    .postgres_user
                    .unwrap_or_else(|| defaults::POSTGRES_USER.into()),
                password: args.postgres_password,
                host: args
                    .postgres_host
                    .unwrap_or_else(|| defaults::POSTGRES_HOST.into()),
                port: args
                    .postgres_port
                    .unwrap_or_else(|| defaults::POSTGRES_PORT.into()),
                database: args.postgres_database,
            },
            graphql_api: GraphQLConfig {
                host: args
                    .graphql_api_host
                    .unwrap_or_else(|| defaults::GRAPHQL_API_HOST.into()),
                port: args
                    .graphql_api_port
                    .unwrap_or_else(|| defaults::GRAPHQL_API_PORT.into()),
            },
        }
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let mut config = IndexerConfig::default();
        let tmp_config: TmpIndexerConfig = serde_yaml::from_str(&contents)?;

        config.upgrade_optionals(tmp_config);
        config.inject_env_vars();

        Ok(config)
    }

    pub fn inject_env_vars(&mut self) {
        let _ = self.fuel_node.inject_env_vars();
        let _ = self.postgres.inject_env_vars();
        let _ = self.graphql_api.inject_env_vars();
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
        let fuel_node_addr = format!("{}:{}", &config.fuel_node.host, &config.fuel_node.port)
            .parse()
            .expect("Failed");

        let manager = SchemaManager::new(&config.postgres.to_string())?;

        Ok(IndexerService {
            fuel_node_addr,
            manager,
            database_url: config.postgres.to_string(),
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
