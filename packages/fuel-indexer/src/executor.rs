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
    time::{sleep, Duration},
};
use tracing::{debug, error, info, warn};
use wasmer::{
    imports, CompilerConfig, Cranelift, FunctionEnv, Instance, Memory, Module,
    RuntimeError, Store, TypedFunction,
};
use wasmer_middlewares::metering::MeteringPoints;

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

/// Run the executor task until the kill switch is flipped, or until some other
/// stop criteria is met.
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

    let start_block = manifest
        .start_block()
        .expect("Failed to detect start_block.");
    let end_block = manifest.end_block();
    if end_block.is_none() {
        warn!("No end_block specified in manifest. Indexer will run forever.");
    }
    let stop_idle_indexers = config.stop_idle_indexers;

    let fuel_node_addr = if config.indexer_net_config {
        manifest
            .fuel_client()
            .map(|x| x.to_string())
            .unwrap_or(config.fuel_node.to_string())
    } else {
        config.fuel_node.to_string()
    };

    let node_block_page_size = config.node_block_page_size;

    let mut next_cursor = if start_block > 1 {
        let decremented = start_block - 1;
        Some(decremented.to_string())
    } else {
        None
    };

    let indexer_uid = manifest.uid();

    info!("Indexer({indexer_uid}) subscribing to Fuel node at {fuel_node_addr}");

    let client = FuelClient::from_str(&fuel_node_addr).unwrap_or_else(|e| {
        panic!("Indexer({indexer_uid}) client node connection failed: {e}.")
    });

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
            let (block_info, cursor) = match retrieve_blocks_from_node(
                &client,
                node_block_page_size,
                &next_cursor,
                end_block,
            )
            .await
            {
                Ok((block_info, cursor)) => (block_info, cursor),
                Err(e) => {
                    error!("Fetching blocks failed: {e:?}",);
                    break;
                }
            };

            let result = executor.handle_events(block_info).await;

            if let Err(e) = result {
                // Run time metering is deterministic. There is no point in retrying.
                if let IndexerError::RunTimeLimitExceededError = e {
                    error!("Indexer executor run time limit exceeded. Giving up. <('.')>. Consider increasing metering points");
                    break;
                }
                error!("Indexer executor failed {e:?}, retrying.");
                match e {
                    IndexerError::SqlxError(sqlx::Error::Database(inner)) => {
                        // sqlx v0.7 let's you determine if this was specifically a unique constraint violation
                        // but sqlx v0.6 does not so we use a best guess.
                        //
                        // TODO: https://github.com/FuelLabs/fuel-indexer/issues/1093
                        if inner.constraint().is_some() {
                            // Just bump the cursor and keep going
                            warn!("Constraint violation. Continuing...");

                            // Try to fetch the page again using same cursor.
                            continue;
                        } else {
                            error!("Database error: {inner}.");
                            sleep(Duration::from_secs(DELAY_FOR_SERVICE_ERROR)).await;
                            retry_count += 1;
                        }
                    }
                    _ => {
                        sleep(Duration::from_secs(DELAY_FOR_SERVICE_ERROR)).await;
                        retry_count += 1;
                    }
                }

                if retry_count < INDEXER_FAILED_CALLS {
                    warn!("Indexer({indexer_uid}) retrying handler after {retry_count} failed attempts.");

                    // Try to fetch the page again using same cursor.
                    continue;
                } else {
                    error!(
                        "Indexer({indexer_uid}) failed after retries, giving up. <('.')>"
                    );
                    break;
                }
            }

            if cursor.is_none() {
                num_empty_block_reqs += 1;

                info!("No new blocks to process, sleeping. zzZZ");
                sleep(Duration::from_secs(DELAY_FOR_EMPTY_PAGE)).await;

                if num_empty_block_reqs == max_empty_block_reqs {
                    error!("No blocks being produced, Indexer({indexer_uid}) giving up. <('.')>");
                    break;
                }
            } else {
                next_cursor = cursor;
                num_empty_block_reqs = 0;
            }

            if kill_switch.load(Ordering::SeqCst) {
                info!("Kill switch flipped, stopping Indexer({indexer_uid}). <('.')>");
                break;
            }

            retry_count = 0;
        }
    }
}

/// Retrieve blocks from a client node.
///
// This was abstracted out of `run_executor` in order to allow for
// use in the benchmarking suite to give consistent timings.
pub async fn retrieve_blocks_from_node(
    client: &FuelClient,
    block_page_size: usize,
    next_cursor: &Option<String>,
    end_block: Option<u64>,
) -> IndexerResult<(Vec<BlockData>, Option<String>)> {
    debug!("Fetching paginated results from {next_cursor:?}");

    let PaginatedResult {
        cursor, results, ..
    } = client
        .full_blocks(PaginationRequest {
            cursor: next_cursor.clone(),
            results: block_page_size,
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
                return Err(IndexerError::EndBlockMet);
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

            let transaction =
                fuel_tx::Transaction::from_bytes(trans.raw_payload.0 .0.as_slice())
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
                    inputs: tx.inputs().iter().map(|i| i.to_owned().into()).collect(),
                    outputs: tx.outputs().iter().map(|o| o.to_owned().into()).collect(),
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
                    contracts_root: <[u8; 32]>::try_from(contracts_root.0 .0.to_owned())
                        .unwrap()
                        .into(),
                    messages_root: <[u8; 32]>::try_from(messages_root.0 .0.to_owned())
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
                id: Bytes32::from(<[u8; 32]>::try_from(block.header.id.0 .0).unwrap()),
                da_height: block.header.da_height.0,
                transactions_count: block.header.transactions_count.0,
                output_messages_count: block.header.output_messages_count.0,
                transactions_root: Bytes32::from(
                    <[u8; 32]>::try_from(block.header.transactions_root.0 .0).unwrap(),
                ),
                output_messages_root: Bytes32::from(
                    <[u8; 32]>::try_from(block.header.output_messages_root.0 .0).unwrap(),
                ),
                height: block.header.height.0,
                prev_root: Bytes32::from(
                    <[u8; 32]>::try_from(block.header.prev_root.0 .0).unwrap(),
                ),
                time: block.header.time.0.to_unix(),
                application_hash: Bytes32::from(
                    <[u8; 32]>::try_from(block.header.application_hash.0 .0).unwrap(),
                ),
            },
            transactions,
        };

        block_info.push(block);
    }

    Ok((block_info, cursor))
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

#[derive(Clone)]
pub struct IndexEnv {
    pub memory: Option<Memory>,
    pub alloc: Option<TypedFunction<u32, u32>>,
    pub dealloc: Option<TypedFunction<(u32, u32), ()>>,
    pub db: Arc<Mutex<Database>>,
}

impl IndexEnv {
    pub async fn new(
        pool: IndexerConnectionPool,
        manifest: &Manifest,
        config: &IndexerConfig,
    ) -> IndexerResult<IndexEnv> {
        let db = Database::new(pool, manifest, config).await;
        Ok(IndexEnv {
            memory: None,
            alloc: None,
            dealloc: None,
            db: Arc::new(Mutex::new(db)),
        })
    }
}

// TODO: https://github.com/FuelLabs/fuel-indexer/issues/1139
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
    /// Create a new `NativeIndexExecutor`.
    pub async fn new(
        manifest: &Manifest,
        pool: IndexerConnectionPool,
        config: &IndexerConfig,
        handle_events_fn: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> F,
    ) -> IndexerResult<Self> {
        let mut db = Database::new(pool.clone(), manifest, config).await;
        let mut conn = pool.acquire().await?;
        let version = fuel_indexer_database::queries::type_id_latest(
            &mut conn,
            manifest.namespace(),
            manifest.identifier(),
        )
        .await?;
        db.load_schema(version).await?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            manifest: manifest.to_owned(),
            handle_events_fn,
        })
    }

    /// Create a new `NativeIndexExecutor`.
    pub async fn create<T: Future<Output = IndexerResult<()>> + Send + 'static>(
        config: &IndexerConfig,
        manifest: &Manifest,
        pool: IndexerConnectionPool,
        handle_events: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> T,
    ) -> IndexerResult<(JoinHandle<()>, ExecutorSource, Arc<AtomicBool>)> {
        let executor =
            NativeIndexExecutor::new(manifest, pool, config, handle_events).await?;
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
    store: Arc<Mutex<Store>>,
    db: Arc<Mutex<Database>>,
    metering_points: Option<u64>,
}

impl WasmIndexExecutor {
    /// Create a new `WasmIndexExecutor`.
    pub async fn new(
        config: &IndexerConfig,
        manifest: &Manifest,
        wasm_bytes: impl AsRef<[u8]>,
        pool: IndexerConnectionPool,
    ) -> IndexerResult<Self> {
        let mut compiler_config = Cranelift::new();

        if let Some(metering_points) = config.metering_points {
            // `Metering` needs to be configured with a limit and a cost
            // function. For each `Operator`, the metering middleware will call
            // the cost function and subtract the cost from the remaining
            // points.
            let metering =
                Arc::new(wasmer_middlewares::Metering::new(metering_points, |_| 1));
            compiler_config.push_middleware(metering);
        }

        let idx_env = IndexEnv::new(pool, manifest, config).await?;
        let db: Arc<Mutex<Database>> = idx_env.db.clone();

        let mut store = Store::new(compiler_config);

        let module = Module::new(&store, &wasm_bytes)?;

        let env = FunctionEnv::new(&mut store, idx_env);
        let mut imports = imports! {};
        for (export_name, export) in ffi::get_exports(&mut store, &env) {
            imports.define("env", &export_name, export.clone());
        }

        let instance = Instance::new(&mut store, &module, &imports)?;

        if !instance
            .exports
            .contains(ffi::MODULE_ENTRYPOINT.to_string())
        {
            return Err(IndexerError::MissingHandler);
        }

        // FunctionEnvMut and SotreMut must be scoped because they can't be used
        // across await
        let version = {
            let mut env_mut = env.clone().into_mut(&mut store);
            let (data_mut, mut store_mut) = env_mut.data_and_store_mut();

            data_mut.memory = Some(instance.exports.get_memory("memory")?.clone());
            data_mut.alloc = Some(
                instance
                    .exports
                    .get_typed_function(&store_mut, "alloc_fn")?,
            );
            data_mut.dealloc = Some(
                instance
                    .exports
                    .get_typed_function(&store_mut, "dealloc_fn")?,
            );

            ffi::get_version(&mut store_mut, &instance)?
        };

        db.lock().await.load_schema(version).await?;

        Ok(WasmIndexExecutor {
            instance,
            _module: module,
            store: Arc::new(Mutex::new(store)),
            db: db.clone(),
            metering_points: config.metering_points,
        })
    }

    /// Restore index from wasm file
    pub async fn from_file(
        p: impl AsRef<Path>,
        config: Option<IndexerConfig>,
        pool: IndexerConnectionPool,
    ) -> IndexerResult<WasmIndexExecutor> {
        let config = config.unwrap_or_default();
        let manifest = Manifest::from_file(p)?;
        let bytes = manifest.module_bytes()?;
        Self::new(&config, &manifest, bytes, pool).await
    }

    /// Create a new `WasmIndexExecutor`.
    pub async fn create(
        config: &IndexerConfig,
        manifest: &Manifest,
        exec_source: ExecutorSource,
        pool: IndexerConnectionPool,
    ) -> IndexerResult<(JoinHandle<()>, ExecutorSource, Arc<AtomicBool>)> {
        let killer = Arc::new(AtomicBool::new(false));

        match &exec_source {
            ExecutorSource::Manifest => match manifest.module() {
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

    /// Returns true if metering is enabled.
    pub fn metering_enabled(&self) -> bool {
        self.metering_points.is_some()
    }

    /// Returns true if metering is enabled metering points are exhausted.
    /// Otherwise returns false.
    pub async fn metering_points_exhausted(&self) -> bool {
        if self.metering_enabled() {
            self.get_remaining_metering_points().await.unwrap()
                == MeteringPoints::Exhausted
        } else {
            false
        }
    }

    /// Returns remaining metering points if metering is enabled. Otherwise, returns None.
    pub async fn get_remaining_metering_points(&self) -> Option<MeteringPoints> {
        if self.metering_enabled() {
            let mut store_guard = self.store.lock().await;
            let result = wasmer_middlewares::metering::get_remaining_points(
                &mut store_guard,
                &self.instance,
            );
            Some(result)
        } else {
            None
        }
    }

    /// Sets the remaining metering points if metering is enabled. Otherwise, returns an error.
    pub async fn set_metering_points(&self, metering_points: u64) -> IndexerResult<()> {
        if self.metering_enabled() {
            let mut store_guard = self.store.lock().await;
            wasmer_middlewares::metering::set_remaining_points(
                &mut store_guard,
                &self.instance,
                metering_points,
            );
            Ok(())
        } else {
            Err(IndexerError::Unknown(
                "Attempting to set metering points when metering is not enables"
                    .to_string(),
            ))
        }
    }
}

#[async_trait]
impl Executor for WasmIndexExecutor {
    /// Trigger a WASM event handler, passing in a serialized event struct.
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()> {
        if let Some(metering_points) = self.metering_points {
            self.set_metering_points(metering_points).await?
        }
        let bytes = serialize(&blocks);

        let mut arg = {
            let mut store_guard = self.store.lock().await;
            ffi::WasmArg::new(
                &mut store_guard,
                &self.instance,
                bytes,
                self.metering_points.is_some(),
            )?
        };

        let fun = {
            let store_guard = self.store.lock().await;
            self.instance.exports.get_typed_function::<(u32, u32), ()>(
                &store_guard,
                ffi::MODULE_ENTRYPOINT,
            )?
        };

        let _ = self.db.lock().await.start_transaction().await?;

        let ptr = arg.get_ptr();
        let len = arg.get_len();

        let res = spawn_blocking({
            let store = self.store.clone();
            move || {
                let mut store_guard =
                    tokio::runtime::Handle::current().block_on(store.lock());
                fun.call(&mut store_guard, ptr, len)
            }
        })
        .await?;

        if let Err(e) = res {
            if self.metering_points_exhausted().await {
                self.db.lock().await.revert_transaction().await?;
                return Err(IndexerError::RunTimeLimitExceededError);
            } else {
                error!("WasmIndexExecutor WASM execution failed: {e:?}.");
                self.db.lock().await.revert_transaction().await?;
                return Err(IndexerError::from(e));
            }
        } else {
            let _ = self.db.lock().await.commit_transaction().await?;
        }

        let mut store_guard = self.store.lock().await;
        arg.drop(&mut store_guard);

        Ok(())
    }
}
