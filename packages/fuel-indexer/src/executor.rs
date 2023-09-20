/// Abstractions for indexer task execution.
use crate::{
    database::Database, ffi, queries::ClientExt, IndexerConfig, IndexerError,
    IndexerResult,
};
use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use fuel_core_client::client::{
    pagination::{PageDirection, PaginatedResult, PaginationRequest},
    schema::block::{Consensus as ClientConsensus, Genesis as ClientGenesis},
    types::TransactionStatus as ClientTransactionStatus,
    FuelClient,
};
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::{
    defaults::*, manifest::Manifest, utils::serialize, WasmIndexerError,
};
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
use tokio::{
    task::spawn_blocking,
    time::{sleep, Duration},
};
use tracing::{debug, error, info, warn};
use wasmer::{
    imports, CompilerConfig, Cranelift, FunctionEnv, Instance, Memory, Module, Store,
    TypedFunction,
};
use wasmer_middlewares::metering::MeteringPoints;

/// Source of the indexer's execution.
#[derive(Debug, Clone)]
pub enum ExecutorSource {
    /// The executor was created from a manifest file.
    Manifest,

    /// The executor was created from indexer bytes stored in the database.
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

impl From<ExecutorSource> for Vec<u8> {
    fn from(source: ExecutorSource) -> Self {
        match source {
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
    mut executor: T,
) -> impl Future<Output = ()> {
    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/286

    let end_block = executor.manifest().end_block();
    let stop_idle_indexers = config.stop_idle_indexers;
    let indexer_uid = executor.manifest().uid();
    let node_block_page_size = config.node_block_page_size;

    let fuel_node_addr = executor
        .manifest()
        .fuel_client()
        .map(|x| x.to_string())
        .unwrap_or(config.fuel_node.to_string());

    // Where should we initially start when fetching blocks from the client?
    let mut cursor = executor.manifest().start_block().map(|x| {
        if x > 1 {
            let decremented = x - 1;
            decremented.to_string()
        } else {
            "0".to_string()
        }
    });

    info!("Indexer({indexer_uid}) subscribing to Fuel node at {fuel_node_addr}");

    let client = FuelClient::from_str(&fuel_node_addr).unwrap_or_else(|e| {
        panic!("Indexer({indexer_uid}) client node connection failed: {e}.")
    });

    if let Some(end_block) = end_block {
        info!("Indexer({indexer_uid}) will stop at block #{end_block}.");
    } else {
        warn!("No end_block specified in the manifest. Indexer({indexer_uid}) will run forever.");
    }

    async move {
        // If we reach an issue that continues to fail, we'll retry a few times before giving up, as
        // we don't want to quit on the first error. But also don't want to waste CPU.
        //
        // Note that this count considers _consecutive_ failed calls.
        let mut consecutive_retries = 0;

        // If we're testing or running on CI, we don't want indexers to run forever. But in production
        // let the indexer service operator decide if they want to stop idle indexers.
        //
        // Maybe we can eventually make this MAX_CONSECUTIVE_EMPTY_BLOCK_RESPONSES value configurable
        //
        // Also note that this count considers _consecutive_ empty block requests.
        let max_empty_block_reqs = if stop_idle_indexers {
            MAX_CONSECUTIVE_EMPTY_BLOCK_RESPONSES
        } else {
            usize::MAX
        };

        // Keep track of how many empty pages we've received from the client.
        let mut num_empty_block_reqs = 0;

        loop {
            // If something else has signaled that this indexer should stop, then stop.
            if executor.kill_switch().load(Ordering::SeqCst) {
                info!("Kill switch flipped, stopping Indexer({indexer_uid}). <('.')>");
                break;
            }

            // Fetch the next page of blocks, and the starting cursor for the subsequent page
            let (block_info, next_cursor, _has_next_page) =
                match retrieve_blocks_from_node(
                    &client,
                    node_block_page_size,
                    &cursor,
                    end_block,
                    &indexer_uid,
                )
                .await
                {
                    Ok((block_info, next_cursor, has_next_page)) => {
                        (block_info, next_cursor, has_next_page)
                    }
                    Err(e) => {
                        if let IndexerError::EndBlockMet = e {
                            info!("Indexer({indexer_uid}) has met its end block; beginning indexer shutdown process.");
                            executor.kill_switch().store(true, Ordering::SeqCst);
                            continue;
                        } else {
                            error!(
                                "Indexer({indexer_uid}) failed to fetch blocks: {e:?}",
                            );
                            sleep(Duration::from_secs(DELAY_FOR_SERVICE_ERROR)).await;
                            continue;
                        }
                    }
                };

            // If our block page request from the client returns empty, we sleep for a bit, and then continue.
            if block_info.is_empty() {
                num_empty_block_reqs += 1;

                info!(
                    "Indexer({indexer_uid}) has no new blocks to process, sleeping zzZZ. (Empty response #{num_empty_block_reqs})"
                );

                if num_empty_block_reqs == max_empty_block_reqs {
                    error!("No blocks being produced after {num_empty_block_reqs} empty responses. Indexer({indexer_uid}) giving up. <('.')>");
                    break;
                }

                // There is no work to do, so we sleep for a bit, then continue without updating our cursor.
                sleep(Duration::from_secs(IDLE_SERVICE_WAIT_SECS)).await;
                continue;
            }

            // The client responded with actual blocks, so attempt to index them.
            let result = executor.handle_events(block_info).await;

            // If the kill switch has been triggered, the executor exits early.
            if executor.kill_switch().load(Ordering::SeqCst) {
                info!("Kill switch flipped, stopping Indexer({indexer_uid}). <('.')>");
                break;
            }

            if let Err(e) = result {
                // Run time metering is deterministic. There is no point in retrying.
                if let IndexerError::RunTimeLimitExceededError = e {
                    error!("Indexer({indexer_uid}) executor run time limit exceeded. Giving up. <('.')>. Consider increasing metering points");
                    break;
                }

                // We don't want to retry forever as that eats resources.
                if consecutive_retries >= INDEXER_FAILED_CALLS {
                    error!(
                        "Indexer({indexer_uid}) failed after too many retries, giving up. <('.')>"
                    );
                    break;
                }

                if let IndexerError::SqlxError(sqlx::Error::Database(inner)) = e {
                    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/1093
                    if inner.constraint().is_some() {
                        // Just bump the cursor and keep going. These errors do not count towards `INDEXER_FAILED_CALLS`
                        warn!("Constraint violation. This is not a retry-able error. Continuing...");
                        cursor = next_cursor;
                        continue;
                    }
                }

                // If we get here, this must be an error that allows us to retry.
                warn!("Indexer({indexer_uid}) retrying handler after {consecutive_retries}/{INDEXER_FAILED_CALLS} failed attempts.");

                consecutive_retries += 1;

                // Since there was some type of error, we're gonna call `retrieve_blocks_from_node` again,
                // with our same cursor.
                continue;
            }

            // If we get a non-empty response, we reset the counter.
            num_empty_block_reqs = 0;

            // If we make it this far, we always go to the next page.
            cursor = next_cursor;

            // Again, check if something else has signaled that this indexer should stop, then stop.
            if executor.kill_switch().load(Ordering::SeqCst) {
                info!("Kill switch flipped, stopping Indexer({indexer_uid}). <('.')>");
                break;
            }

            // Since we had successful call, we reset the retry count.
            consecutive_retries = 0;
        }
    }
}

/// Retrieve blocks from a client node.
///
/// This was abstracted out of `run_executor` in order to allow for use in the benchmarking suite
/// to give consistent timings.
///
/// If there is an issue fetching blocks, we return an empty cursor and an empty list of blocks, then
/// run_executor will determine whether or not we should sleep for a bit and try again.
pub async fn retrieve_blocks_from_node(
    client: &FuelClient,
    block_page_size: usize,
    cursor: &Option<String>,
    end_block: Option<u32>,
    indexer_uid: &str,
) -> IndexerResult<(Vec<BlockData>, Option<String>, bool)> {
    // Let's check if we need less blocks than block_page_size.
    let page_size = if let (Some(start), Some(end)) = (cursor, end_block) {
        if let Ok(start) = start.parse::<u32>() {
            if start >= end {
                return Err(IndexerError::EndBlockMet);
            }

            std::cmp::min((end - start) as usize, block_page_size)
        } else {
            block_page_size
        }
    } else {
        block_page_size
    };

    debug!("Fetching paginated results from {cursor:?}");

    let PaginatedResult {
        cursor,
        results,
        has_next_page,
        ..
    } = client
        .full_blocks(PaginationRequest {
            cursor: cursor.clone(),
            results: page_size,
            direction: PageDirection::Forward,
        })
        .await
        .unwrap_or_else(|e| {
            error!("Indexer({indexer_uid}) failed to retrieve blocks: {e:?}");
            // Setting an empty cursor will cause the indexer to sleep for a bit and try again.
            PaginatedResult {
                cursor: None,
                results: vec![],
                has_next_page: false,
                has_previous_page: false,
            }
        });

    let chain_id = client.chain_info().await?.consensus_parameters.chain_id;

    let mut block_info = Vec::new();
    for block in results.into_iter() {
        let producer: Option<Bytes32> = block.block_producer().map(|pk| pk.hash());

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

            let transaction: fuel_tx::Transaction =
                fuel_tx::Transaction::from_bytes(trans.raw_payload.0 .0.as_slice())
                    .expect("Bad transaction.");

            let id = transaction.id(&chain_id);

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
                            key: <[u8; 32]>::from(*x.key()).into(),
                            value: <[u8; 32]>::from(*x.value()).into(),
                        })
                        .collect(),
                    inputs: tx.inputs().iter().map(|i| i.to_owned().into()).collect(),
                    outputs: tx.outputs().iter().map(|o| o.to_owned().into()).collect(),
                    witnesses: tx.witnesses().to_vec(),
                    salt: <[u8; 32]>::from(*tx.salt()).into(),
                    metadata: None,
                }),
                ClientTransaction::Script(tx) => Transaction::Script(Script {
                    gas_price: *tx.gas_price(),
                    gas_limit: *tx.gas_limit(),
                    maturity: *tx.maturity(),
                    script: (*tx.script().clone()).to_vec(),
                    script_data: (*tx.script_data().clone()).to_vec(),
                    inputs: tx.inputs().iter().map(|i| i.to_owned().into()).collect(),
                    outputs: tx.outputs().iter().map(|o| o.to_owned().into()).collect(),
                    witnesses: tx.witnesses().to_vec(),
                    receipts_root: <[u8; 32]>::from(*tx.receipts_root()).into(),
                    metadata: None,
                }),
                ClientTransaction::Mint(tx) => Transaction::Mint(Mint {
                    tx_pointer: tx.tx_pointer().to_owned().into(),
                    outputs: tx.outputs().iter().map(|o| o.to_owned().into()).collect(),
                    metadata: None,
                }),
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
                    chain_config_hash: <[u8; 32]>::from(
                        chain_config_hash.to_owned().0 .0,
                    )
                    .into(),
                    coins_root: <[u8; 32]>::from(coins_root.0 .0.to_owned()).into(),
                    contracts_root: <[u8; 32]>::from(contracts_root.0 .0.to_owned())
                        .into(),
                    messages_root: <[u8; 32]>::from(messages_root.0 .0.to_owned()).into(),
                })
            }
            ClientConsensus::PoAConsensus(poa) => Consensus::PoA(PoA {
                signature: <[u8; 64]>::from(poa.signature.0 .0.to_owned()).into(),
            }),
        };

        // TODO: https://github.com/FuelLabs/fuel-indexer/issues/286
        let block = BlockData {
            height: block.header.height.clone().into(),
            id: Bytes32::from(<[u8; 32]>::from(block.id.0 .0)),
            producer,
            time: block.header.time.0.to_unix(),
            consensus,
            header: Header {
                id: block.header.id.into(),
                da_height: block.header.da_height.0,
                transactions_count: block.header.transactions_count.into(),
                message_receipt_count: block.header.message_receipt_count.into(),
                transactions_root: block.header.transactions_root.into(),
                message_receipt_root: block.header.message_receipt_root.into(),
                height: block.header.height.into(),
                prev_root: block.header.prev_root.into(),
                time: block.header.time.0.to_unix(),
                application_hash: block.header.application_hash.into(),
            },
            transactions,
        };

        block_info.push(block);
    }

    Ok((block_info, cursor, has_next_page))
}

/// Executors are responsible for the actual indexing of data.
///
/// Executors can either be WASM modules or native Rust functions.
#[async_trait]
pub trait Executor
where
    Self: Sized,
{
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()>;

    fn manifest(&self) -> &Manifest;

    fn kill_switch(&self) -> &Arc<AtomicBool>;
}

/// WASM indexer runtime environment responsible for fetching/saving data to and from the database.
#[derive(Clone)]
pub struct IndexEnv {
    /// Memory allocated to this runtime.
    pub memory: Option<Memory>,

    /// Allocator function used to allocate memory for calls that fetch items from the database.
    pub alloc: Option<TypedFunction<u32, u32>>,

    /// Deallocator function used to deallocate memory after the associated `WasmArg` is dropped.
    pub dealloc: Option<TypedFunction<(u32, u32), ()>>,

    /// Reference to the connected database.
    pub db: Arc<Mutex<Database>>,

    /// Kill switch for this indexer. When true, the indexer service indicated
    /// that the indexer is being terminated.
    pub kill_switch: Arc<AtomicBool>,
}

impl IndexEnv {
    /// Create a new `IndexEnv`.
    pub async fn new(
        pool: IndexerConnectionPool,
        manifest: &Manifest,
        config: &IndexerConfig,
        kill_switch: Arc<AtomicBool>,
    ) -> IndexerResult<IndexEnv> {
        let db = Database::new(pool, manifest, config).await;
        Ok(IndexEnv {
            memory: None,
            alloc: None,
            dealloc: None,
            db: Arc::new(Mutex::new(db)),
            kill_switch,
        })
    }
}

/// Native executors differ from WASM executors in that they are not sandboxed; they are merely a
/// set of native Rust functions that (run/execute/are spawned) directly from the indexer service
/// process.
pub struct NativeIndexExecutor<F>
where
    F: Future<Output = IndexerResult<()>> + Send,
{
    /// Reference to the connected database.
    db: Arc<Mutex<Database>>,

    /// Manifest of the indexer.
    manifest: Manifest,

    /// Function that handles events.
    handle_events_fn: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> F,

    /// Kill switch. When set to true, the indexer must stop execution.
    kill_switch: Arc<AtomicBool>,
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
        let kill_switch = Arc::new(AtomicBool::new(false));
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            manifest: manifest.to_owned(),
            handle_events_fn,
            kill_switch,
        })
    }

    /// Create a new `NativeIndexExecutor`.
    pub async fn create(
        config: &IndexerConfig,
        manifest: &Manifest,
        pool: IndexerConnectionPool,
        handle_events: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> F,
    ) -> IndexerResult<Self> {
        NativeIndexExecutor::new(manifest, pool.clone(), config, handle_events).await
    }
}

#[async_trait]
impl<F> Executor for NativeIndexExecutor<F>
where
    F: Future<Output = IndexerResult<()>> + Send,
{
    /// Handle events for  native executor.
    async fn handle_events(&mut self, blocks: Vec<BlockData>) -> IndexerResult<()> {
        self.db.lock().await.start_transaction().await?;
        let res = (self.handle_events_fn)(blocks, self.db.clone()).await;
        let uid = self.manifest.uid();
        if let Err(e) = res {
            error!("NativeIndexExecutor({uid}) handle_events failed: {e:?}.");
            self.db.lock().await.revert_transaction().await?;
            return Err(IndexerError::NativeExecutionRuntimeError);
        } else {
            // Do not commit if kill switch has been triggered.
            if self.kill_switch.load(Ordering::SeqCst) {
                self.db.lock().await.revert_transaction().await?;
            } else {
                self.db.lock().await.commit_transaction().await?;
            }
        }
        Ok(())
    }

    fn kill_switch(&self) -> &Arc<AtomicBool> {
        &self.kill_switch
    }

    fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

/// WASM executors are the primary means of execution.
///
/// WASM executors contain a WASM module that is instantiated and executed by the indexer service on a
/// virtually infinite tokio task loop. These executors are responsible for allocating/deallocating their
/// own memory for calls in and out of the runtime.
#[derive(Debug)]
pub struct WasmIndexExecutor {
    /// Associated wasmer module instance.
    instance: Instance,

    /// Associated wasmer module.
    _module: Module,

    /// Associated wasmer store.
    store: Arc<Mutex<Store>>,

    /// Reference to the connected database.
    db: Arc<Mutex<Database>>,

    /// Number of metering points to use for this executor.
    metering_points: Option<u64>,

    /// Manifest of the indexer.
    manifest: Manifest,

    /// Kill switch. When set to true, the indexer must stop execution.
    kill_switch: Arc<AtomicBool>,
}

impl WasmIndexExecutor {
    /// Create a new `WasmIndexExecutor`.
    pub async fn new(
        config: &IndexerConfig,
        manifest: &Manifest,
        wasm_bytes: impl AsRef<[u8]>,
        pool: IndexerConnectionPool,
        schema_version: String,
    ) -> IndexerResult<Self> {
        let mut compiler_config = Cranelift::new();

        if let Some(metering_points) = config.metering_points {
            // `Metering` needs to be configured with a limit and a cost
            // function. For each `Operator`, the metering middleware will call
            // the cost function and subtract the cost from the remaining points.
            let metering =
                Arc::new(wasmer_middlewares::Metering::new(metering_points, |_| 1));
            compiler_config.push_middleware(metering);
        }

        let kill_switch = Arc::new(AtomicBool::new(false));

        let idx_env = IndexEnv::new(pool, manifest, config, kill_switch.clone()).await?;

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

        // FunctionEnvMut and StoreMut must be scoped because they can't
        // be used across await
        {
            let mut env_mut = env.clone().into_mut(&mut store);
            let (data_mut, mut store_mut) = env_mut.data_and_store_mut();

            let schema_version_from_wasm = ffi::get_version(&mut store_mut, &instance)?;

            if schema_version_from_wasm != schema_version {
                return Err(IndexerError::SchemaVersionMismatch(format!(
                    "Schema version from WASM {} does not match schema version from database {}",
                    schema_version_from_wasm, schema_version
                )));
            }

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
        }

        db.lock().await.load_schema(schema_version).await?;

        Ok(WasmIndexExecutor {
            instance,
            _module: module,
            store: Arc::new(Mutex::new(store)),
            db: db.clone(),
            metering_points: config.metering_points,
            manifest: manifest.clone(),
            kill_switch,
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
        let schema_version = manifest.graphql_schema_content()?.version().to_string();
        Self::new(&config, &manifest, bytes, pool, schema_version).await
    }

    /// Create a new `WasmIndexExecutor`.
    pub async fn create(
        config: &IndexerConfig,
        manifest: &Manifest,
        pool: IndexerConnectionPool,
        schema_version: String,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> IndexerResult<Self> {
        let uid = manifest.uid();

        match WasmIndexExecutor::new(config, manifest, wasm_bytes, pool, schema_version)
            .await
        {
            Ok(executor) => Ok(executor),
            Err(e) => {
                error!("Could not instantiate WasmIndexExecutor({uid}): {e:?}.");
                Err(IndexerError::WasmExecutionInstantiationError)
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
        if blocks.is_empty() {
            return Ok(());
        }

        if let Some(metering_points) = self.metering_points {
            self.set_metering_points(metering_points).await?
        }
        let bytes = serialize(&blocks);
        let uid = self.manifest.uid();

        let fun = {
            let store_guard = self.store.lock().await;
            self.instance.exports.get_typed_function::<(u32, u32), ()>(
                &store_guard,
                ffi::MODULE_ENTRYPOINT,
            )?
        };

        let _ = self.db.lock().await.start_transaction().await?;

        let res = spawn_blocking({
            let store = self.store.clone();
            let instance = self.instance.clone();
            let metering_enabled = self.metering_enabled();
            move || {
                let store_guard =
                    tokio::runtime::Handle::current().block_on(store.lock());
                let mut arg =
                    ffi::WasmArg::new(store_guard, instance, bytes, metering_enabled)
                        .unwrap();

                let ptr = arg.get_ptr();
                let len = arg.get_len();

                fun.call(&mut arg.store(), ptr, len)
            }
        })
        .await?;

        if let Err(e) = res {
            if self.metering_points_exhausted().await {
                self.db.lock().await.revert_transaction().await?;
                return Err(IndexerError::RunTimeLimitExceededError);
            } else {
                if let Some(e) = e.downcast_ref::<WasmIndexerError>() {
                    match e {
                        // Termination due to kill switch is an expected behavior.
                        WasmIndexerError::KillSwitch => {
                            info!("Indexer({uid}) WASM execution terminated: {e}.")
                        }
                        _ => {
                            error!("Indexer({uid}) WASM execution failed: {e}.")
                        }
                    }
                } else {
                    error!("Indexer({uid}) WASM execution failed: {e:?}.");
                };
                self.db.lock().await.revert_transaction().await?;
                return Err(IndexerError::from(e));
            }
        } else {
            // Do not commit if kill switch has been triggered.
            if self.kill_switch.load(Ordering::SeqCst) {
                self.db.lock().await.revert_transaction().await?;
            } else {
                self.db.lock().await.commit_transaction().await?;
            }
        }

        Ok(())
    }

    fn kill_switch(&self) -> &Arc<AtomicBool> {
        &self.kill_switch
    }

    fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}
