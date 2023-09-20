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

        self.tasks
            .spawn(crate::executor::run_executor(&self.config, executor));
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
                std::cmp::max(start, last)
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
