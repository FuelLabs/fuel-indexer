use crate::{
    database::Database, ffi, queries::ClientExt, IndexerConfig, IndexerError,
    IndexerResult,
};
use async_std::{
    fs::File,
    io::ReadExt,
    sync::{Arc, Mutex},
};
use async_trait::async_trait;
use fuel_core_client::client::{
    schema::block::{Consensus as ClientConsensus, Genesis as ClientGenesis},
    types::TransactionStatus as ClientTransactionStatus,
    FuelClient, PageDirection, PaginatedResult, PaginationRequest,
};
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::{defaults::*, manifest::Manifest, utils::serialize};
use fuel_indexer_types::{
    fuel::{field::*, *},
    scalar::{Bytes32, HexString},
};
use fuel_tx::UniqueIdentifier;
use fuel_vm::prelude::Deserializable;
use fuel_vm::state::ProgramState as ClientProgramState;
use futures::Future;
use itertools::Itertools;
use std::{
    marker::{Send, Sync},
    path::Path,
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
};
use thiserror::Error;
use tokio::{
    task::{spawn_blocking, JoinHandle},
    time::{sleep, timeout, Duration},
};
use tracing::{debug, error, info, warn};
use wasmer::{
    imports, Instance, LazyInit, Memory, Module, NativeFunc, RuntimeError, Store,
    WasmerEnv,
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

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
            ExecutorSource::Registry(b) => b,
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

// Run the executor task until the kill switch is flipped, or until some other
// stop criteria is met.
//
// In general the logic in this function isn't very idiomatic, but that's because
// types in `fuel_core_client` don't compile to WASM.
pub fn run_executor<T: 'static + Executor + Send + Sync>(
    config: &IndexerConfig,
    manifest: &Manifest,
    mut executor: T,
    kill_switch: Arc<AtomicBool>,
) -> impl Future<Output = ()> {
    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/286

    let start_block = manifest.start_block.expect("Failed to detect start_block.");
    let end_block = manifest.end_block;
    if end_block.is_none() {
        warn!("No end_block specified in manifest. Indexer will run forever.");
    }
    let stop_idle_indexers = config.stop_idle_indexers;

    let fuel_node_addr = if config.indexer_net_config {
        manifest
            .fuel_client
            .clone()
            .unwrap_or(config.fuel_node.to_string())
    } else {
        config.fuel_node.to_string()
    };

    let mut next_cursor = if start_block > 1 {
        let decremented = start_block - 1;
        Some(decremented.to_string())
    } else {
        None
    };

    info!("Subscribing to Fuel node at {fuel_node_addr}");

    let client = FuelClient::from_str(&fuel_node_addr)
        .unwrap_or_else(|e| panic!("Node connection failed: {e}."));

    async move {
        let mut retry_count = 0;

        // If we're testing or running on CI, we don't want indexers to run forever. But in production
        // let the index operators decide if they want to stop idle indexers. Maybe we can eventually
        // make this MAX_EMPTY_BLOCK_REQUESTS value configurable
        let max_empty_block_reqs = if stop_idle_indexers {
            MAX_EMPTY_BLOCK_REQUESTS
        } else {
            usize::MAX
        };
        let mut num_empty_block_reqs = 0;

        loop {
            debug!("Fetching paginated results from {next_cursor:?}");

            let PaginatedResult {
                cursor, results, ..
            } = client
                .full_blocks(PaginationRequest {
                    cursor: next_cursor.clone(),
                    results: NODE_GRAPHQL_PAGE_SIZE,
                    direction: PageDirection::Forward,
                })
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to retrieve blocks: {e}");
                    PaginatedResult {
                        cursor: None,
                        results: vec![],
                        has_next_page: false,
                        has_previous_page: false,
                    }
                });

            let mut block_info = Vec::new();
            for block in results.into_iter() {
                if let Some(end_block) = end_block {
                    if block.header.height.0 > end_block {
                        info!("Stopping indexer at the specified end_block: {end_block}");
                        break;
                    }
                }

                let producer = block
                    .block_producer()
                    .map(|pk| Bytes32::from(<[u8; 32]>::try_from(pk.hash()).unwrap()));

                let mut transactions = Vec::new();

                for trans in block.transactions {
                    let receipts = trans
                        .receipts
                        .unwrap_or_default()
                        .into_iter()
                        .map(TryInto::try_into)
                        .try_collect()
                        .expect("Bad receipts.");

                    let status = trans.status.expect("Bad transaction status.");
                    // NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
                    let status = match status.try_into().unwrap() {
                        ClientTransactionStatus::Success {
                            block_id,
                            time,
                            program_state,
                        } => {
                            let program_state = program_state.map(|p| match p {
                                ClientProgramState::Return(w) => ProgramState {
                                    return_type: ReturnType::Return,
                                    data: HexString::from(w.to_le_bytes().to_vec()),
                                },
                                ClientProgramState::ReturnData(d) => ProgramState {
                                    return_type: ReturnType::ReturnData,
                                    data: HexString::from(d.to_vec()),
                                },
                                ClientProgramState::Revert(w) => ProgramState {
                                    return_type: ReturnType::Revert,
                                    data: HexString::from(w.to_le_bytes().to_vec()),
                                },
                                // Either `cargo watch` complains that this is unreachable, or `clippy` complains
                                // that all patterns are not matched. These other program states are only used in
                                // debug modes.
                                #[allow(unreachable_patterns)]
                                _ => unreachable!("Bad program state."),
                            });
                            TransactionStatus::Success {
                                block: block_id.parse().expect("Bad block height."),
                                time: time.to_unix() as u64,
                                program_state,
                            }
                        }
                        ClientTransactionStatus::Failure {
                            block_id,
                            time,
                            reason,
                            program_state,
                        } => {
                            let program_state = program_state.map(|p| match p {
                                ClientProgramState::Return(w) => ProgramState {
                                    return_type: ReturnType::Return,
                                    data: HexString::from(w.to_le_bytes().to_vec()),
                                },
                                ClientProgramState::ReturnData(d) => ProgramState {
                                    return_type: ReturnType::ReturnData,
                                    data: HexString::from(d.to_vec()),
                                },
                                ClientProgramState::Revert(w) => ProgramState {
                                    return_type: ReturnType::Revert,
                                    data: HexString::from(w.to_le_bytes().to_vec()),
                                },
                                // Either `cargo watch` complains that this is unreachable, or `clippy` complains
                                // that all patterns are not matched. These other program states are only used in
                                // debug modes.
                                #[allow(unreachable_patterns)]
                                _ => unreachable!("Bad program state."),
                            });
                            TransactionStatus::Failure {
                                block: block_id.parse().expect("Bad block ID."),
                                time: time.to_unix() as u64,
                                program_state,
                                reason,
                            }
                        }
                        ClientTransactionStatus::Submitted { submitted_at } => {
                            TransactionStatus::Submitted {
                                submitted_at: submitted_at.to_unix() as u64,
                            }
                        }
                        ClientTransactionStatus::SqueezedOut { reason } => {
                            TransactionStatus::SqueezedOut { reason }
                        }
                    };

                    let transaction = fuel_tx::Transaction::from_bytes(
                        trans.raw_payload.0 .0.as_slice(),
                    )
                    .expect("Bad transaction.");

                    let id = transaction.id();

                    let transaction = match transaction {
                        ClientTransaction::Create(tx) => Transaction::Create(Create {
                            gas_price: *tx.gas_price(),
                            gas_limit: *tx.gas_limit(),
                            maturity: *tx.maturity(),
                            bytecode_length: *tx.bytecode_length(),
                            bytecode_witness_index: *tx.bytecode_witness_index(),
                            storage_slots: tx
                                .storage_slots()
                                .iter()
                                .map(|x| StorageSlot {
                                    key: <[u8; 32]>::try_from(*x.key())
                                        .expect("Could not convert key to bytes")
                                        .into(),
                                    value: <[u8; 32]>::try_from(*x.value())
                                        .expect("Could not convert key to bytes")
                                        .into(),
                                })
                                .collect(),
                            inputs: tx
                                .inputs()
                                .iter()
                                .map(|i| i.to_owned().into())
                                .collect(),
                            outputs: tx
                                .outputs()
                                .iter()
                                .map(|o| o.to_owned().into())
                                .collect(),
                            witnesses: tx.witnesses().to_vec(),
                            salt: <[u8; 32]>::try_from(*tx.salt())
                                .expect("Could not convert key to bytes")
                                .into(),
                            metadata: None,
                        }),
                        _ => Transaction::default(),
                    };

                    let tx_data = TransactionData {
                        receipts,
                        status,
                        transaction,
                        id,
                    };

                    transactions.push(tx_data);
                }

                // TODO: https://github.com/FuelLabs/fuel-indexer/issues/286
                let consensus = match &block.consensus {
                    ClientConsensus::Unknown => Consensus::Unknown,
                    ClientConsensus::Genesis(g) => {
                        let ClientGenesis {
                            chain_config_hash,
                            coins_root,
                            contracts_root,
                            messages_root,
                        } = g.to_owned();

                        Consensus::Genesis(Genesis {
                            chain_config_hash: <[u8; 32]>::try_from(
                                chain_config_hash.to_owned().0 .0,
                            )
                            .unwrap()
                            .into(),
                            coins_root: <[u8; 32]>::try_from(coins_root.0 .0.to_owned())
                                .unwrap()
                                .into(),
                            contracts_root: <[u8; 32]>::try_from(
                                contracts_root.0 .0.to_owned(),
                            )
                            .unwrap()
                            .into(),
                            messages_root: <[u8; 32]>::try_from(
                                messages_root.0 .0.to_owned(),
                            )
                            .unwrap()
                            .into(),
                        })
                    }
                    ClientConsensus::PoAConsensus(poa) => Consensus::PoA(PoA {
                        signature: <[u8; 64]>::try_from(poa.signature.0 .0.to_owned())
                            .unwrap()
                            .into(),
                    }),
                };

                // TODO: https://github.com/FuelLabs/fuel-indexer/issues/286
                let block = BlockData {
                    height: block.header.height.clone().into(),
                    id: Bytes32::from(<[u8; 32]>::try_from(block.id.0 .0).unwrap()),
                    producer,
                    time: block.header.time.0.to_unix(),
                    consensus,
                    header: Header {
                        id: Bytes32::from(
                            <[u8; 32]>::try_from(block.header.id.0 .0).unwrap(),
                        ),
                        da_height: block.header.da_height.0,
                        transactions_count: block.header.transactions_count.0,
                        output_messages_count: block.header.output_messages_count.0,
                        transactions_root: Bytes32::from(
                            <[u8; 32]>::try_from(block.header.transactions_root.0 .0)
                                .unwrap(),
                        ),
                        output_messages_root: Bytes32::from(
                            <[u8; 32]>::try_from(block.header.output_messages_root.0 .0)
                                .unwrap(),
                        ),
                        height: block.header.height.0,
                        prev_root: Bytes32::from(
                            <[u8; 32]>::try_from(block.header.prev_root.0 .0).unwrap(),
                        ),
                        time: block.header.time.0.to_unix(),
                        application_hash: Bytes32::from(
                            <[u8; 32]>::try_from(block.header.application_hash.0 .0)
                                .unwrap(),
                        ),
                    },
                    transactions,
                };

                block_info.push(block);
            }

            let result = executor.handle_events(block_info).await;

            if let Err(e) = result {
                error!("Indexer executor failed {e:?}, retrying.");
                match e {
                    IndexerError::SqlxError(sqlx::Error::Database(inner)) => {
                        // sqlx v0.7 let's you determine if this was specifically a unique constraint violation
                        // but sqlx v0.6 does not so we use a best guess.
                        if inner.constraint().is_some() {
                            warn!("Constraint violation. Continuing...");
                            next_cursor = cursor;
                            continue;
                        } else {
                            error!("Database error: {inner}.");
                            retry_count += 1;
                        }
                    }
                    _ => {
                        sleep(Duration::from_secs(DELAY_FOR_SERVICE_ERROR)).await;
                        retry_count += 1;
                    }
                }

                if retry_count < INDEXER_FAILED_CALLS {
                    warn!("Retrying handler after {retry_count} failed attempts.");
                    continue;
                } else {
                    error!("Indexer failed after retries, giving up. <('.')>");
                    break;
                }
            }

            if cursor.is_none() {
                info!("No new blocks to process, sleeping.");
                sleep(Duration::from_secs(DELAY_FOR_EMPTY_PAGE)).await;

                num_empty_block_reqs += 1;

                if num_empty_block_reqs == max_empty_block_reqs {
                    error!("No blocks being produced, giving up. <('.')>");
                    break;
                }
            } else {
                next_cursor = cursor;
                num_empty_block_reqs = 0;
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
    pub async fn new(pool: IndexerConnectionPool) -> IndexerResult<IndexEnv> {
        let db = Arc::new(Mutex::new(Database::new(pool).await?));
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

pub struct NativeIndexExecutor<F>
where
    F: Future<Output = IndexerResult<()>> + Send,
{
    db: Arc<Mutex<Database>>,
    #[allow(unused)]
    manifest: Manifest,
    handle_events_fn: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> F,
}

impl<F> NativeIndexExecutor<F>
where
    F: Future<Output = IndexerResult<()>> + Send,
{
    pub async fn new(
        manifest: &Manifest,
        pool: IndexerConnectionPool,
        handle_events_fn: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> F,
    ) -> IndexerResult<Self> {
        let db = Arc::new(Mutex::new(Database::new(pool).await?));
        db.lock().await.load_schema(manifest, None).await?;
        Ok(Self {
            db,
            manifest: manifest.to_owned(),
            handle_events_fn,
        })
    }

    pub async fn create<T: Future<Output = IndexerResult<()>> + Send + 'static>(
        config: &IndexerConfig,
        manifest: &Manifest,
        pool: IndexerConnectionPool,
        handle_events: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> T,
    ) -> IndexerResult<(JoinHandle<()>, ExecutorSource, Arc<AtomicBool>)> {
        let executor = NativeIndexExecutor::new(manifest, pool, handle_events).await?;
        let kill_switch = Arc::new(AtomicBool::new(false));
        let handle = tokio::spawn(run_executor(
            config,
            manifest,
            executor,
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
            error!("NativeIndexExecutor handle_events failed: {e}.");
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
    #[allow(unused)]
    timeout: u64,
}

impl WasmIndexExecutor {
    pub async fn new(
        config: &IndexerConfig,
        manifest: &Manifest,
        wasm_bytes: impl AsRef<[u8]>,
        pool: IndexerConnectionPool,
    ) -> IndexerResult<Self> {
        let store = Store::new(&Universal::new(compiler()).engine());
        let module = Module::new(&store, &wasm_bytes)?;

        let mut import_object = imports! {};

        let mut env = IndexEnv::new(pool).await?;
        let exports = ffi::get_exports(&env, &store);

        import_object.register("env", exports);

        let instance = Instance::new(&module, &import_object)?;
        env.init_with_instance(&instance)?;
        env.db
            .lock()
            .await
            .load_schema(manifest, Some(&instance))
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
            timeout: config.indexer_handler_timeout,
        })
    }

    /// Restore index from wasm file
    pub async fn from_file(
        p: impl AsRef<Path>,
        config: Option<IndexerConfig>,
        pool: IndexerConnectionPool,
    ) -> IndexerResult<Self> {
        let config = config.unwrap_or_default();
        let manifest = Manifest::from_file(p)?;
        let bytes = manifest.module_bytes()?;
        Self::new(&config, &manifest, bytes, pool).await
    }

    pub async fn create(
        config: &IndexerConfig,
        manifest: &Manifest,
        exec_source: ExecutorSource,
        pool: IndexerConnectionPool,
    ) -> IndexerResult<(JoinHandle<()>, ExecutorSource, Arc<AtomicBool>)> {
        let killer = Arc::new(AtomicBool::new(false));

        match &exec_source {
            ExecutorSource::Manifest => match &manifest.module {
                crate::Module::Wasm(ref module) => {
                    let mut bytes = Vec::<u8>::new();
                    let mut file = File::open(module).await?;
                    file.read_to_end(&mut bytes).await?;

                    let executor =
                        WasmIndexExecutor::new(config, manifest, bytes.clone(), pool)
                            .await?;
                    let handle = tokio::spawn(run_executor(
                        config,
                        manifest,
                        executor,
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
                    WasmIndexExecutor::new(config, manifest, bytes, pool).await?;
                let handle = tokio::spawn(run_executor(
                    config,
                    manifest,
                    executor,
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

        let _ = self.db.lock().await.start_transaction().await?;

        let ptr = arg.get_ptr();
        let len = arg.get_len();

        let res = timeout(
            Duration::from_secs(self.timeout),
            spawn_blocking(move || fun.call(ptr, len)),
        )
        .await;

        if let Err(e) = res {
            error!("WasmIndexExecutor handle_events timed out: {e:?}.");
            let _ = self.db.lock().await.revert_transaction().await?;
            return Err(IndexerError::from(e));
        } else {
            let inner = res.unwrap();
            if let Err(e) = inner {
                error!("WasmIndexExecutor handle_events failed: {e:?}.");
                self.db.lock().await.revert_transaction().await?;
                return Err(IndexerError::from(e));
            }
            let _ = self.db.lock().await.commit_transaction().await?;
        }
        Ok(())
    }
}
