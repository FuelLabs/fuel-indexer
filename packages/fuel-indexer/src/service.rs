use crate::{
    executor::{ExecutorSource, NativeIndexExecutor, WasmIndexExecutor},
    Database, IndexerConfig, IndexerError, IndexerResult, Manifest,
};
use async_std::sync::{Arc, Mutex};
use fuel_indexer_database::{
    queries, types::IndexerAssetType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_lib::{defaults, utils::ServiceRequest};
use fuel_indexer_schema::db::manager::SchemaManager;
use fuel_indexer_types::fuel::BlockData;
use futures::{
    stream::{FuturesUnordered, StreamExt},
    Future,
};
use std::collections::HashMap;
use std::marker::Send;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{
    sync::mpsc::Receiver,
    task::JoinHandle,
    time::{sleep, Duration},
};
use tracing::{debug, error, info, warn};

/// Primary service used to run one or many indexers.
pub struct IndexerService {
    /// Indexer service configuration.
    config: IndexerConfig,

    /// Connection pool used to connect to the database.
    pool: IndexerConnectionPool,

    /// Schema manager used to manage the database schema.
    manager: SchemaManager,

    /// Handles to the spawned indexers.
    handles: HashMap<String, JoinHandle<()>>,

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
            handles: HashMap::default(),
            killers: HashMap::default(),
            rx,
        })
    }

    /// Register new indexers to the `IndexerService`, from a `Manifest`.
    pub async fn register_indexer_from_manifest(
        &mut self,
        mut manifest: Manifest,
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

        let (handle, exec_source, killer) = WasmIndexExecutor::create(
            &self.config,
            &manifest,
            ExecutorSource::Manifest,
            self.pool.clone(),
            schema_version,
        )
        .await?;

        let mut items = vec![
            (IndexerAssetType::Wasm, exec_source.into()),
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
        self.handles.insert(manifest.uid(), handle);
        self.killers.insert(manifest.uid(), killer);

        Ok(())
    }

    /// Register pre-existing indexers from the database.
    pub async fn register_indexers_from_registry(&mut self) -> IndexerResult<()> {
        let mut conn = self.pool.acquire().await?;
        let indices = queries::all_registered_indexers(&mut conn).await?;
        for index in indices {
            let assets = queries::latest_assets_for_indexer(&mut conn, &index.id).await?;
            let mut manifest = Manifest::try_from(&assets.manifest.bytes)?;

            let start_block = get_start_block(&mut conn, &manifest).await.unwrap_or(1);
            manifest.set_start_block(start_block);

            if let Ok((handle, _module_bytes, killer)) = WasmIndexExecutor::create(
                &self.config,
                &manifest,
                ExecutorSource::Registry(assets.wasm.bytes),
                self.pool.clone(),
                assets.schema.digest,
            )
            .await
            {
                info!("Registered Indexer({})", manifest.uid());
                self.handles.insert(manifest.uid(), handle);
                self.killers.insert(manifest.uid(), killer);
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
        let (handle, _module_bytes, killer) = NativeIndexExecutor::<T>::create(
            &self.config,
            &manifest,
            self.pool.clone(),
            handle_events,
        )
        .await?;

        info!("Registered NativeIndex({})", uid);

        self.handles.insert(uid.clone(), handle);
        self.killers.insert(uid, killer);
        Ok(())
    }

    /// Kick it off!
    pub async fn run(self) {
        let IndexerService {
            handles,
            rx,
            pool,
            config,
            killers,
            ..
        } = self;

        let futs = Arc::new(Mutex::new(FuturesUnordered::from_iter(
            handles.into_values(),
        )));

        let node_block_page_size = config.node_block_page_size;
        let client_uri = config.fuel_node.uri();
        let mut conn = pool.acquire().await.unwrap();

        let start_block_height = queries::last_block_height_for_stored_blocks(&mut conn)
            .await
            .unwrap();

        tokio::spawn(async move {
            let mut cursor = Some(start_block_height.to_string());

            info!(
                "Block fetcher: starting from Block#{}",
                start_block_height + 1
            );

            let client =
                fuel_core_client::client::FuelClient::new(client_uri.to_string())
                    .unwrap_or_else(|e| panic!("Client node connection failed: {e}."));

            loop {
                info!("Block fetcher: cursor {:?}", cursor);
                // Fetch the next page of blocks, and the starting cursor for the subsequent page
                let (block_info, next_cursor, _has_next_page) =
                    match crate::executor::retrieve_blocks_from_node(
                        &client,
                        node_block_page_size,
                        &cursor,
                        None,
                        "block_fetcher",
                    )
                    .await
                    {
                        Ok((block_info, next_cursor, _has_next_page)) => {
                            if !block_info.is_empty() {
                                let first = block_info[0].height;
                                let last = block_info.last().unwrap().height;
                                info!(
                                    "Block fetcher: retrieved blocks: {}-{}. Has next page? {}",
                                    first, last, _has_next_page
                                );
                            } else {
                                info!("Block fetcher: no new blocks.")
                            }
                            (block_info, next_cursor, _has_next_page)
                        }
                        Err(e) => {
                            error!("Indexer() failed to fetch blocks: {e:?}",);
                            sleep(Duration::from_secs(
                                fuel_indexer_lib::defaults::DELAY_FOR_SERVICE_ERROR,
                            ))
                            .await;
                            continue;
                        }
                    };

                // Blocks must be in order, and there can be no missing blocks.
                // This is enforced when saving to the database by a trigger. If
                // `save_blockdata` succeeds, all is well.
                fuel_indexer_database::queries::save_blockdata(&mut conn, &block_info)
                    .await
                    .unwrap();

                cursor = next_cursor;
            }
        })
        .await
        .unwrap();

        let _ = tokio::spawn(create_service_task(
            rx,
            config.clone(),
            pool.clone(),
            futs.clone(),
            killers,
        ))
        .await
        .unwrap();

        while let Some(fut) = futs.lock().await.next().await {
            info!("Retired a future {fut:?}");
        }
    }
}

/// Create a tokio task used to listen to service messages primarily coming from the web API.
async fn create_service_task(
    mut rx: Receiver<ServiceRequest>,
    config: IndexerConfig,
    pool: IndexerConnectionPool,
    futs: Arc<Mutex<FuturesUnordered<JoinHandle<()>>>>,
    mut killers: HashMap<String, Arc<AtomicBool>>,
) -> IndexerResult<()> {
    loop {
        let futs = futs.lock().await;
        match rx.try_recv() {
            Ok(service_request) => match service_request {
                ServiceRequest::Reload(request) => {
                    let mut conn = pool.acquire().await?;

                    match queries::get_indexer_id(
                        &mut conn,
                        &request.namespace,
                        &request.identifier,
                    )
                    .await
                    {
                        Ok(id) => {
                            let assets =
                                queries::latest_assets_for_indexer(&mut conn, &id)
                                    .await?;
                            let mut manifest =
                                Manifest::try_from(&assets.manifest.bytes)?;

                            let start_block =
                                get_start_block(&mut conn, &manifest).await?;
                            manifest.set_start_block(start_block);

                            match WasmIndexExecutor::create(
                                &config,
                                &manifest,
                                ExecutorSource::Registry(assets.wasm.bytes),
                                pool.clone(),
                                assets.schema.digest,
                            )
                            .await
                            {
                                Ok((handle, _module_bytes, killer)) => {
                                    futs.push(handle);

                                    if let Some(killer_for_prev_executor) =
                                        killers.insert(manifest.uid(), killer)
                                    {
                                        let uid = manifest.uid();
                                        info!("Indexer({uid}) was replaced. Stopping previous version of Indexer({uid}).");
                                        killer_for_prev_executor
                                            .store(true, Ordering::SeqCst);
                                    }
                                }
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

                    if let Some(killer) = killers.remove(&uid) {
                        killer.store(true, Ordering::SeqCst);
                    } else {
                        warn!("Stop Indexer: No indexer with the name Indexer({uid})");
                    }
                }
            },
            Err(e) => {
                debug!("No service request to handle: {e:?}.");
                sleep(Duration::from_secs(defaults::IDLE_SERVICE_WAIT_SECS)).await;
            }
        }
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
                // if the last processed block is N, we want to resume from N+1
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
