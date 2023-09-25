use crate::{
    executor::{NativeIndexExecutor, WasmIndexExecutor},
    Database, Executor, IndexerConfig, IndexerError, IndexerResult, Manifest,
};
use async_std::sync::{Arc, Mutex};
use async_std::{fs::File, io::ReadExt};
use fuel_indexer_database::{
    queries, types::IndexerAssetType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_lib::utils::ServiceRequest;
use fuel_indexer_schema::db::manager::SchemaManager;
use fuel_indexer_types::fuel::BlockData;
use futures::Future;
use std::collections::HashMap;
use std::marker::Send;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info, warn};

/// Primary service used to run one or many indexers.
pub struct IndexerService {
    /// Indexer service configuration.
    config: IndexerConfig,

    /// Connection pool used to connect to the database.
    pool: IndexerConnectionPool,

    /// Schema manager used to manage the database schema.
    manager: SchemaManager,

    /// Tasks for the spawned indexers.
    tasks: tokio::task::JoinSet<()>,

    /// Channel used to receive `ServiceRequest`s.
    rx: Receiver<ServiceRequest>,

    /// Killers used to stop the spawned indexers.
    killers: HashMap<String, Arc<AtomicBool>>,
}

impl IndexerService {
    /// Create a new `IndexerService`.
    pub async fn new(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        rx: Receiver<ServiceRequest>,
    ) -> IndexerResult<IndexerService> {
        let manager = SchemaManager::new(pool.clone());

        Ok(IndexerService {
            config,
            pool,
            manager,
            killers: HashMap::default(),
            tasks: tokio::task::JoinSet::new(),
            rx,
        })
    }

    /// Register new indexers to the `IndexerService`, from a `Manifest`.
    pub async fn register_indexer_from_manifest(
        &mut self,
        mut manifest: Manifest,
        remove_data: bool,
    ) -> IndexerResult<()> {
        if let Some(killer) = self.killers.get(&manifest.uid()) {
            killer.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        let mut conn = self.pool.acquire().await?;

        let indexer_exists = (queries::get_indexer_id(
            &mut conn,
            manifest.namespace(),
            manifest.identifier(),
        )
        .await)
            .is_ok();
        if indexer_exists {
            if !self.config.replace_indexer {
                return Err(IndexerError::Unknown(format!(
                    "Indexer({}.{}) already exists.",
                    manifest.namespace(),
                    manifest.identifier()
                )));
            } else if let Err(e) = queries::remove_indexer(
                &mut conn,
                manifest.namespace(),
                manifest.identifier(),
                remove_data,
            )
            .await
            {
                error!(
                    "Failed to remove Indexer({}.{}): {e}",
                    manifest.namespace(),
                    manifest.identifier()
                );
                queries::revert_transaction(&mut conn).await?;
                return Err(e.into());
            }
        }

        let _indexer = queries::register_indexer(
            &mut conn,
            manifest.namespace(),
            manifest.identifier(),
            None,
        )
        .await?;

        let schema = manifest.graphql_schema_content()?;
        let schema_version = schema.version().to_string();
        let schema_bytes = Vec::<u8>::from(&schema);

        self.manager
            .new_schema(
                manifest.namespace(),
                manifest.identifier(),
                schema,
                manifest.execution_source(),
                &mut conn,
            )
            .await?;

        let start_block = get_start_block(&mut conn, &manifest).await?;
        manifest.set_start_block(start_block);

        let wasm_bytes = match manifest.module() {
            crate::Module::Wasm(ref module) => {
                let mut bytes = Vec::<u8>::new();
                let mut file = File::open(module).await?;
                file.read_to_end(&mut bytes).await?;
                bytes
            }
            crate::Module::Native => {
                return Err(IndexerError::NativeExecutionInstantiationError)
            }
        };

        let executor = WasmIndexExecutor::create(
            &self.config,
            &manifest,
            self.pool.clone(),
            schema_version,
            wasm_bytes.clone(),
        )
        .await?;

        let mut items = vec![
            (IndexerAssetType::Wasm, wasm_bytes),
            (IndexerAssetType::Manifest, manifest.clone().into()),
            (IndexerAssetType::Schema, schema_bytes),
        ];

        while let Some((asset_type, bytes)) = items.pop() {
            info!(
                "Registering Asset({asset_type:?}) for Indexer({})",
                manifest.uid()
            );

            {
                queries::register_indexer_asset(
                    &mut conn,
                    manifest.namespace(),
                    manifest.identifier(),
                    bytes,
                    asset_type,
                    None,
                )
                .await?;
            }
        }
        info!(
            "Registered Indexer({}.{})",
            manifest.namespace(),
            manifest.identifier()
        );

        self.start_executor(executor);

        Ok(())
    }

    /// Register pre-existing indexers from the database.
    pub async fn register_indexers_from_registry(&mut self) -> IndexerResult<()> {
        let mut conn = self.pool.acquire().await?;
        let indices = queries::all_registered_indexers(&mut conn).await?;
        for index in indices {
            let assets = queries::indexer_assets(&mut conn, &index.id).await?;
            let mut manifest = Manifest::try_from(&assets.manifest.bytes)?;

            let start_block = get_start_block(&mut conn, &manifest).await.unwrap_or(1);
            manifest.set_start_block(start_block);

            if let Ok(executor) = WasmIndexExecutor::create(
                &self.config,
                &manifest,
                self.pool.clone(),
                assets.schema.digest,
                assets.wasm.bytes,
            )
            .await
            {
                info!("Registered Indexer({})", manifest.uid());

                self.start_executor(executor);
            } else {
                error!(
                    "Failed to register Indexer({}) from registry.",
                    manifest.uid()
                );
            }
        }

        Ok(())
    }

    /// Register a native indexer to the `IndexerService`, from a `Manifest`.
    pub async fn register_native_indexer<
        T: Future<Output = IndexerResult<()>> + Send + 'static,
    >(
        &mut self,
        mut manifest: Manifest,
        handle_events: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> T,
    ) -> IndexerResult<()> {
        let mut conn = self.pool.acquire().await?;
        let _index = queries::register_indexer(
            &mut conn,
            manifest.namespace(),
            manifest.identifier(),
            None,
        )
        .await?;

        self.manager
            .new_schema(
                manifest.namespace(),
                manifest.identifier(),
                manifest.graphql_schema_content()?,
                manifest.execution_source(),
                &mut conn,
            )
            .await?;

        let start_block = get_start_block(&mut conn, &manifest).await.unwrap_or(1);
        manifest.set_start_block(start_block);

        let uid = manifest.uid();
        let executor = NativeIndexExecutor::<T>::create(
            &self.config,
            &manifest,
            self.pool.clone(),
            handle_events,
        )
        .await?;

        info!("Registered NativeIndex({})", uid);

        self.start_executor(executor);

        Ok(())
    }

    /// Kick it off! Run the indexer service loop, listening to service messages primarily coming from the web server.
    pub async fn run(mut self) -> IndexerResult<()> {
        loop {
            tokio::select! {
                // Calling join_next will remove finished tasks from the set.
                Some(Err(e)) = self.tasks.join_next() => {
                    error!("Error retiring indexer task {e}");
                }
                Some(service_request) = self.rx.recv() => {
                    match service_request {
                        ServiceRequest::Reload(request) => {
                            let mut conn = self.pool.acquire().await?;

                            match queries::get_indexer_id(
                                &mut conn,
                                &request.namespace,
                                &request.identifier,
                            )
                            .await
                            {
                                Ok(id) => {
                                    let assets =
                                        queries::indexer_assets(&mut conn, &id)
                                            .await?;
                                    let mut manifest =
                                        Manifest::try_from(&assets.manifest.bytes)?;

                                    let start_block =
                                        get_start_block(&mut conn, &manifest).await?;
                                    manifest.set_start_block(start_block);

                                    if let Some(killer_for_prev_executor) =
                                        self.killers.remove(&manifest.uid())
                                    {
                                        let uid = manifest.uid();
                                        info!("Indexer({uid}) is being replaced. Stopping previous version of Indexer({uid}).");
                                        killer_for_prev_executor
                                            .store(true, Ordering::SeqCst);
                                    }

                                    match WasmIndexExecutor::create(
                                        &self.config,
                                        &manifest,
                                        self.pool.clone(),
                                        assets.schema.digest,
                                        assets.wasm.bytes,
                                    )
                                    .await
                                    {
                                        Ok(executor) => self.start_executor(executor),
                                        Err(e) => {
                                            error!(
                                                "Failed to reload Indexer({}.{}): {e:?}",
                                                &request.namespace, &request.identifier
                                            );
                                            return Ok(());
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to find Indexer({}.{}): {}",
                                        &request.namespace, &request.identifier, e
                                    );

                                    continue;
                                }
                            }
                        }
                        ServiceRequest::Stop(request) => {
                            let uid = format!("{}.{}", request.namespace, request.identifier);

                            if let Some(killer) = self.killers.remove(&uid) {
                                killer.store(true, Ordering::SeqCst);
                            } else {
                                warn!(
                                    "Stop Indexer: No indexer with the name Indexer({uid})"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Spawn and register a tokio::task running the Executor loop, as well as
    // the kill switch and the abort handle.
    fn start_executor<T: 'static + Executor + Send + Sync>(&mut self, executor: T) {
        let uid = executor.manifest().uid();

        self.killers
            .insert(uid.clone(), executor.kill_switch().clone());

        self.tasks.spawn(crate::executor::run_executor(
            &self.config,
            self.pool.clone(),
            executor,
        ));
    }
}

/// Determine the starting block for this indexer.
pub async fn get_start_block(
    conn: &mut IndexerConnection,
    manifest: &Manifest,
) -> Result<u32, IndexerError> {
    match &manifest.resumable() {
        Some(resumable) => {
            let last = queries::last_block_height_for_indexer(
                conn,
                manifest.namespace(),
                manifest.identifier(),
            )
            .await?;
            let start = manifest.start_block().unwrap_or(last);
            let block = if *resumable {
                // If the last processed block is N, we want to resume from N+1.
                // A database trigger prevents the indexer from processing the
                // same block twice.
                std::cmp::max(start, last + 1)
            } else {
                start
            };

            let action = if *resumable { "Resuming" } else { "Starting" };

            info!("{action} Indexer({}) from block {block}", manifest.uid());
            Ok(block)
        }
        None => {
            let block = manifest.start_block().unwrap_or(1);
            info!("Starting Indexer({}) from block {block}", manifest.uid());
            Ok(block)
        }
    }
}

/// Create a tokio task for retrieving blocks from Fuel Node and saving them in
/// the database.
pub(crate) async fn create_block_sync_task(
    config: IndexerConfig,
    pool: IndexerConnectionPool,
) {
    let task_id = "Block Sync";

    let mut conn = pool.acquire().await.unwrap();

    let last_height = queries::last_block_height_for_stored_blocks(&mut conn)
        .await
        .unwrap_or_else(|_| panic!("{task_id} was unable to determine the last block height for stored blocks."));

    let mut cursor = Some(last_height.to_string());

    info!("{task_id}: starting from Block#{}", last_height + 1);

    let client =
        fuel_core_client::client::FuelClient::new(config.fuel_node.uri().to_string())
            .unwrap_or_else(|e| panic!("Client node connection failed: {e}."));

    loop {
        // Get the next page of blocks, and the starting cursor for the subsequent page
        let (block_info, next_cursor, _has_next_page) =
            match crate::executor::retrieve_blocks_from_node(
                &client,
                config.block_page_size,
                &cursor,
                None,
                task_id,
            )
            .await
            {
                Ok((block_info, next_cursor, _has_next_page)) => {
                    if !block_info.is_empty() {
                        let first = block_info[0].height;
                        let last = block_info.last().unwrap().height;
                        info!("{task_id}: retrieved blocks: {}-{}.", first, last);
                    }
                    (block_info, next_cursor, _has_next_page)
                }
                Err(e) => {
                    error!("{task_id}: failed to retrieve blocks: {e:?}");
                    tokio::time::sleep(std::time::Duration::from_secs(
                        fuel_indexer_lib::defaults::DELAY_FOR_SERVICE_ERROR,
                    ))
                    .await;
                    continue;
                }
            };

        if block_info.is_empty() {
            info!("{task_id}: no new blocks to process, sleeping zzZZ.");
            tokio::time::sleep(std::time::Duration::from_secs(
                fuel_indexer_lib::defaults::IDLE_SERVICE_WAIT_SECS,
            ))
            .await;
        } else {
            // Blocks must be in order, and there can be no missing blocks. This
            // is enforced when saving to the database by a trigger. If
            // `save_blockdata` succeeds, all is well.
            fuel_indexer_database::queries::save_block_data(&mut conn, &block_info)
                .await
                .unwrap_or_else(|_| panic!("{task_id} was unable to save block data."));

            cursor = next_cursor;
        }
    }
}
