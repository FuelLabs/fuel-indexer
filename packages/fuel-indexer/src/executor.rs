use crate::ffi;
use crate::{database::Database, IndexerError, IndexerResult, Manifest};
use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use fuel_indexer_schema::utils::serialize;
use fuel_indexer_types::abi::BlockData;
use futures::Future;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use tokio::task::spawn_blocking;
use tracing::error;
use wasmer::{
    imports, Instance, LazyInit, Memory, Module, NativeFunc, RuntimeError, Store,
    WasmerEnv,
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

use async_std::{fs::File, io::ReadExt};
use chrono::{TimeZone, Utc};
use fuel_gql_client::client::{
    types::{TransactionResponse, TransactionStatus as GqlTransactionStatus},
    FuelClient, PageDirection, PaginatedResult, PaginationRequest,
};
use fuel_indexer_lib::{
    config::FuelNodeConfig,
    defaults::{
        DELAY_FOR_EMPTY_PAGE, DELAY_FOR_SERVICE_ERR, INDEX_FAILED_CALLS,
        MAX_EMPTY_BLOCK_REQUESTS,
    },
};
use fuel_indexer_types::{
    abi::TransactionData,
    tx::{TransactionStatus, TxId},
    Bytes32,
};
use std::marker::{Send, Sync};
use std::str::FromStr;
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};
use tracing::{debug, info};

fn compiler() -> Cranelift {
    Cranelift::default()
}

#[derive(Debug, Clone)]
pub enum ExecutorSource {
    Manifest,
    Registry(Vec<u8>),
}

impl AsRef<[u8]> for ExecutorSource {
    fn as_ref(&self) -> &[u8] {
        match self {
            ExecutorSource::Manifest => &[],
            ExecutorSource::Registry(b) => &b,
        }
    }
}

impl ExecutorSource {
    pub fn to_vec(self) -> Vec<u8> {
        match self {
            ExecutorSource::Manifest => vec![],
            ExecutorSource::Registry(bytes) => bytes,
        }
    }
}

pub fn run_executor<T: 'static + Executor + Send + Sync>(
    fuel_node_addr: &str,
    mut executor: T,
    start_block: Option<u64>,
    kill_switch: Arc<AtomicBool>,
) -> impl Future<Output = ()> {
    let start_block_value = start_block.unwrap_or(1);
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
        let mut empty_block_reqs = 0;

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
                .unwrap_or_else(|e| {
                    error!("Failed to retrieve blocks: {}", e);
                    PaginatedResult {
                        cursor: None,
                        results: vec![],
                        has_next_page: false,
                        has_previous_page: false,
                    }
                });

            debug!("Processing {} results", results.len());

            let mut block_info = Vec::new();
            for block in results.into_iter().rev() {
                let producer = block.block_producer().map(|pk| pk.hash());

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
                    producer,
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
                if retry_count < INDEX_FAILED_CALLS {
                    continue;
                } else {
                    error!("Indexer failed after retries, giving up. <('.')>");
                    break;
                }
            }

            if cursor.is_none() {
                info!("No new blocks to process, sleeping.");
                sleep(Duration::from_secs(DELAY_FOR_EMPTY_PAGE)).await;

                empty_block_reqs += 1;

                if empty_block_reqs == MAX_EMPTY_BLOCK_REQUESTS {
                    error!("No blocks being produced, giving up. <('.')>");
                    break;
                }
            } else {
                next_cursor = cursor;
                empty_block_reqs = 0;
            }

            if kill_switch.load(Ordering::SeqCst) {
                break;
            }

            retry_count = 0;
        }
    }
}

#[async_trait]
pub trait Executor
where
    Self: Sized,
{
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()>;
}

#[derive(Error, Debug)]
pub enum TxError {
    #[error("WASM Runtime Error {0:?}")]
    WasmRuntimeError(#[from] RuntimeError),
}

#[derive(WasmerEnv, Clone)]
pub struct IndexEnv {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    #[wasmer(export(name = "alloc_fn"))]
    alloc: LazyInit<NativeFunc<u32, u32>>,
    #[wasmer(export(name = "dealloc_fn"))]
    dealloc: LazyInit<NativeFunc<(u32, u32), ()>>,
    pub db: Arc<Mutex<Database>>,
}

impl IndexEnv {
    pub async fn new(db_conn: String) -> IndexerResult<IndexEnv> {
        let db = Arc::new(Mutex::new(Database::new(&db_conn).await?));
        Ok(IndexEnv {
            memory: Default::default(),
            alloc: Default::default(),
            dealloc: Default::default(),
            db,
        })
    }
}

// TODO: Use mutex
unsafe impl<F: Future<Output = IndexerResult<()>> + Send> Sync
    for NativeIndexExecutor<F>
{
}
unsafe impl<F: Future<Output = IndexerResult<()>> + Send> Send
    for NativeIndexExecutor<F>
{
}

#[allow(dead_code)]
pub struct NativeIndexExecutor<F>
where
    F: Future<Output = IndexerResult<()>> + Send,
{
    db: Arc<Mutex<Database>>,
    manifest: Manifest,
    handle_events_fn: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> F,
}

impl<F> NativeIndexExecutor<F>
where
    F: Future<Output = IndexerResult<()>> + Send,
{
    pub async fn new(
        db_conn: &str,
        manifest: Manifest,
        handle_events_fn: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> F,
    ) -> IndexerResult<Self> {
        let db = Arc::new(Mutex::new(Database::new(db_conn).await?));
        db.lock().await.load_schema(&manifest, None).await?;
        Ok(Self {
            db,
            manifest,
            handle_events_fn,
        })
    }

    pub async fn create<T: Future<Output = IndexerResult<()>> + Send + 'static>(
        db_url: &str,
        fuel_node: &FuelNodeConfig,
        manifest: Manifest,
        handle_events: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> T,
    ) -> IndexerResult<(JoinHandle<()>, ExecutorSource, Arc<AtomicBool>)> {
        let start_block = manifest.start_block;
        let executor = NativeIndexExecutor::new(db_url, manifest, handle_events).await?;
        let kill_switch = Arc::new(AtomicBool::new(false));
        let handle = tokio::spawn(run_executor(
            &fuel_node.to_string(),
            executor,
            start_block,
            kill_switch.clone(),
        ));
        Ok((handle, ExecutorSource::Manifest, kill_switch))
    }
}

#[async_trait]
impl<F> Executor for NativeIndexExecutor<F>
where
    F: Future<Output = IndexerResult<()>> + Send,
{
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()> {
        self.db.lock().await.start_transaction().await?;
        let res = (self.handle_events_fn)(blocks, self.db.clone()).await;
        if let Err(e) = res {
            error!("NativeIndexExecutor handle_events failed: {}.", e);
            self.db.lock().await.revert_transaction().await?;
            return Err(IndexerError::NativeExecutionRuntimeError);
        } else {
            self.db.lock().await.commit_transaction().await?;
        }
        Ok(())
    }
}

/// Responsible for loading a single indexer module, triggering events.
#[derive(Debug)]
pub struct WasmIndexExecutor {
    instance: Instance,
    _module: Module,
    _store: Store,
    db: Arc<Mutex<Database>>,
}

impl WasmIndexExecutor {
    pub async fn new(
        db_conn: String,
        manifest: Manifest,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> IndexerResult<Self> {
        let store = Store::new(&Universal::new(compiler()).engine());
        let module = Module::new(&store, &wasm_bytes)?;

        let mut import_object = imports! {};

        let mut env = IndexEnv::new(db_conn).await?;
        let exports = ffi::get_exports(&env, &store);
        import_object.register("env", exports);

        let instance = Instance::new(&module, &import_object)?;
        env.init_with_instance(&instance)?;
        env.db
            .lock()
            .await
            .load_schema(&manifest, Some(&instance))
            .await?;

        if !instance
            .exports
            .contains(ffi::MODULE_ENTRYPOINT.to_string())
        {
            return Err(IndexerError::MissingHandler);
        }

        Ok(WasmIndexExecutor {
            instance,
            _module: module,
            _store: store,
            db: env.db.clone(),
        })
    }

    /// Restore index from wasm file
    pub async fn from_file(db_conn: String, manifest_path: &Path) -> IndexerResult<Self> {
        let manifest = Manifest::from_file(manifest_path)?;
        let bytes = manifest.module_bytes()?;
        Self::new(db_conn, manifest, bytes).await
    }

    pub async fn create(
        fuel_node: &FuelNodeConfig,
        db_url: &str,
        manifest: &Manifest,
        exec_source: ExecutorSource,
    ) -> IndexerResult<(JoinHandle<()>, ExecutorSource, Arc<AtomicBool>)> {
        let killer = Arc::new(AtomicBool::new(false));
        match &exec_source {
            ExecutorSource::Manifest => match &manifest.module {
                crate::Module::Wasm(ref module) => {
                    let mut bytes = Vec::<u8>::new();
                    let mut file = File::open(module).await?;
                    file.read_to_end(&mut bytes).await?;

                    let executor = WasmIndexExecutor::new(
                        db_url.to_string(),
                        manifest.to_owned(),
                        bytes.clone(),
                    )
                    .await?;
                    let handle = tokio::spawn(run_executor(
                        &fuel_node.to_string(),
                        executor,
                        manifest.start_block,
                        killer.clone(),
                    ));

                    Ok((handle, ExecutorSource::Registry(bytes), killer))
                }
                crate::Module::Native => {
                    Err(IndexerError::NativeExecutionInstantiationError)
                }
            },
            ExecutorSource::Registry(bytes) => {
                let executor =
                    WasmIndexExecutor::new(db_url.into(), manifest.to_owned(), bytes)
                        .await?;
                let handle = tokio::spawn(run_executor(
                    &fuel_node.to_string(),
                    executor,
                    manifest.start_block,
                    killer.clone(),
                ));

                Ok((handle, exec_source, killer))
            }
        }
    }
}

#[async_trait]
impl Executor for WasmIndexExecutor {
    /// Trigger a WASM event handler, passing in a serialized event struct.
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()> {
        let bytes = serialize(&blocks);
        let arg = ffi::WasmArg::new(&self.instance, bytes)?;

        let fun = self
            .instance
            .exports
            .get_native_function::<(u32, u32), ()>(ffi::MODULE_ENTRYPOINT)?;

        self.db.lock().await.start_transaction().await?;

        let ptr = arg.get_ptr();
        let len = arg.get_len();

        let res = spawn_blocking(move || fun.call(ptr, len)).await?;

        if let Err(e) = res {
            error!("WasmIndexExecutor handle_events failed: {}.", e.message());
            let frames = e.trace();
            for (i, frame) in frames.iter().enumerate() {
                println!(
                    "Frame #{}: {:?}::{:?}",
                    i,
                    frame.module_name(),
                    frame.function_name()
                );
            }

            self.db.lock().await.revert_transaction().await?;
            return Err(IndexerError::RuntimeError(e));
        } else {
            self.db.lock().await.commit_transaction().await?;
        }
        Ok(())
    }
}
