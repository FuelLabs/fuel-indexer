use crate::{
    executor::{ExecutorSource, NativeIndexExecutor, WasmIndexExecutor},
    Database, IndexerConfig, IndexerResult, Manifest,
};
use async_std::sync::{Arc, Mutex};
use fuel_indexer_database::{queries, types::IndexAssetType, IndexerConnectionPool};
use fuel_indexer_lib::{defaults, utils::ServiceRequest};
use fuel_indexer_schema::db::manager::SchemaManager;
use fuel_indexer_types::abi::BlockData;
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

pub struct IndexerService {
    config: IndexerConfig,
    pool: IndexerConnectionPool,
    manager: SchemaManager,
    database_url: String,
    handles: HashMap<String, JoinHandle<()>>,
    rx: Option<Receiver<ServiceRequest>>,
    killers: HashMap<String, Arc<AtomicBool>>,
}

impl IndexerService {
    pub async fn new(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        rx: Option<Receiver<ServiceRequest>>,
    ) -> IndexerResult<IndexerService> {
        let database_url = config.database.to_string();

        let manager = SchemaManager::new(pool.clone());

        Ok(IndexerService {
            config,
            pool,
            manager,
            database_url,
            handles: HashMap::default(),
            killers: HashMap::default(),
            rx,
        })
    }

    pub async fn register_index_from_manifest(
        &mut self,
        manifest: Manifest,
    ) -> IndexerResult<()> {
        let database_url = self.database_url.clone();
        let mut conn = self.pool.acquire().await?;
        let index =
            queries::register_index(&mut conn, &manifest.namespace, &manifest.identifier)
                .await?;
        let schema = manifest.graphql_schema()?;
        let schema_bytes = schema.as_bytes().to_vec();

        self.manager
            .new_schema(
                &manifest.namespace,
                &manifest.identifier,
                &schema,
                &mut conn,
            )
            .await?;

        let (handle, exec_source, killer) = WasmIndexExecutor::create(
            &self.config.fuel_node.clone(),
            &database_url,
            &manifest,
            ExecutorSource::Manifest,
            self.config.stop_idle_indexers,
        )
        .await?;

        let mut items = vec![
            (IndexAssetType::Wasm, exec_source.to_vec()),
            (IndexAssetType::Manifest, manifest.to_bytes()?),
            (IndexAssetType::Schema, schema_bytes),
        ];

        while let Some((asset_type, bytes)) = items.pop() {
            info!(
                "Registering Asset({:?}) for Index({})",
                asset_type,
                index.uid()
            );

            {
                queries::register_index_asset(
                    &mut conn,
                    &manifest.namespace,
                    &manifest.identifier,
                    bytes,
                    asset_type,
                )
                .await?;
            }
        }

        info!("Registered Index({})", &manifest.uid());
        self.handles.insert(manifest.uid(), handle);
        self.killers.insert(manifest.uid(), killer);

        Ok(())
    }

    pub async fn register_indices_from_registry(&mut self) -> IndexerResult<()> {
        let mut conn = self.pool.acquire().await?;
        let indices = queries::registered_indices(&mut conn).await?;
        for index in indices {
            let assets = queries::latest_assets_for_index(&mut conn, &index.id).await?;
            let manifest = Manifest::from_slice(&assets.manifest.bytes)?;

            let (handle, _module_bytes, killer) = WasmIndexExecutor::create(
                &self.config.fuel_node,
                &self.config.database.to_string(),
                &manifest,
                ExecutorSource::Registry(assets.wasm.bytes),
                self.config.stop_idle_indexers,
            )
            .await?;

            info!("Registered Index({})", manifest.uid());
            self.handles.insert(manifest.uid(), handle);
            self.killers.insert(manifest.uid(), killer);
        }

        Ok(())
    }

    pub async fn register_native_index<
        T: Future<Output = IndexerResult<()>> + Send + 'static,
    >(
        &mut self,
        manifest: Manifest,
        handle_events: fn(Vec<BlockData>, Arc<Mutex<Database>>) -> T,
    ) -> IndexerResult<()> {
        let mut conn = self.pool.acquire().await?;
        let _index =
            queries::register_index(&mut conn, &manifest.namespace, &manifest.identifier)
                .await?;
        let schema = manifest.graphql_schema()?;
        let _schema_bytes = schema.as_bytes().to_vec();

        self.manager
            .new_schema(
                &manifest.namespace,
                &manifest.identifier,
                &schema,
                &mut conn,
            )
            .await?;

        let uid = manifest.uid();
        let (handle, _module_bytes, killer) = NativeIndexExecutor::<T>::create(
            &self.database_url,
            &self.config.fuel_node,
            manifest,
            self.config.stop_idle_indexers,
            handle_events,
        )
        .await?;

        info!("Registered NativeIndex({})", uid);

        self.handles.insert(uid.clone(), handle);
        self.killers.insert(uid, killer);
        Ok(())
    }

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

        tokio::spawn(create_service_task(
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

async fn create_service_task(
    rx: Option<Receiver<ServiceRequest>>,
    config: IndexerConfig,
    pool: IndexerConnectionPool,
    futs: Arc<Mutex<FuturesUnordered<JoinHandle<()>>>>,
    mut killers: HashMap<String, Arc<AtomicBool>>,
) {
    if let Some(mut rx) = rx {
        loop {
            let futs = futs.lock().await;
            match rx.try_recv() {
                Ok(service_request) => match service_request {
                    ServiceRequest::AssetReload(request) => {
                        let mut conn = pool
                            .acquire()
                            .await
                            .expect("Failed to acquire connection from pool");

                        match queries::index_id_for(
                            &mut conn,
                            &request.namespace,
                            &request.identifier,
                        )
                        .await
                        {
                            Ok(id) => {
                                let assets =
                                    queries::latest_assets_for_index(&mut conn, &id)
                                        .await
                                        .expect("Could not get latest assets for index");

                                let manifest: Manifest =
                                    serde_yaml::from_slice(&assets.manifest.bytes)
                                        .expect("Failed to deserialize manifest");

                                let (handle, _module_bytes, killer) = WasmIndexExecutor::create(
                                    &config.fuel_node,
                                    &config.database.to_string(),
                                    &manifest,
                                    ExecutorSource::Registry(assets.wasm.bytes),
                                    config.stop_idle_indexers,
                                )
                                .await
                                .expect(
                                    "Failed to spawn executor from index asset registry",
                                );

                                futs.push(handle);
                                killers.insert(manifest.uid(), killer);
                            }
                            Err(e) => {
                                error!(
                                    "Failed to find Index({}.{}): {}",
                                    &request.namespace, &request.identifier, e
                                );

                                continue;
                            }
                        }
                    }
                    ServiceRequest::IndexStop(request) => {
                        let uid = format!("{}.{}", request.namespace, request.identifier);

                        if let Some(killer) = killers.remove(&uid) {
                            killer.store(true, Ordering::SeqCst);
                        } else {
                            warn!("Stop Indexer: No indexer with the name Index({uid})");
                        }
                    }
                    ServiceRequest::IndexRevert(request) => {
                        let uid = format!("{}.{}", request.namespace, request.identifier);

                        if let Some(killer) = killers.get(&uid) {
                            killer.store(true, Ordering::SeqCst);
                        } else {
                            warn!(
                                "Revert Indexer: No indexer with the name Index({uid})"
                            );
                        }

                        let mut conn = pool
                            .acquire()
                            .await
                            .expect("Failed to acquire connection from pool");

                        match queries::penultimate_asset_for_index(
                            &mut conn,
                            &request.namespace,
                            &request.identifier,
                            IndexAssetType::Wasm,
                        )
                        .await
                        {
                            Ok(penultimate_asset) => {
                                let _ = queries::start_transaction(&mut conn)
                                    .await
                                    .expect("Failed to start transaction");

                                let latest_assets =
                                    queries::latest_assets_for_index(&mut conn, &penultimate_asset.id)
                                        .await
                                        .expect("Could not get latest assets for index");

                                if let Err(_e) = queries::remove_asset_by_version(
                                    &mut conn,
                                    &latest_assets.manifest.id,
                                    &latest_assets.wasm.version,
                                    IndexAssetType::Wasm,
                                )
                                .await
                                {
                                    queries::revert_transaction(&mut conn)
                                        .await
                                        .expect("Failed to revert transaction");
                                } else {
                                    queries::commit_transaction(&mut conn)
                                        .await
                                        .expect("Failed to commit transaction");
                                }
                                

                                let manifest: Manifest =
                                    serde_yaml::from_slice(&penultimate_asset.bytes)
                                        .expect("Failed to deserialize manifest");

                                let (handle, _module_bytes, killer) = WasmIndexExecutor::create(
                                    &config.fuel_node,
                                    &config.database.to_string(),
                                    &manifest,
                                    ExecutorSource::Registry(penultimate_asset.bytes),
                                    config.stop_idle_indexers,
                                )
                                .await
                                .expect(
                                    "Failed to spawn executor from index asset registry",
                                );

                                futs.push(handle);
                                killers.insert(manifest.uid(), killer);
                            }
                            Err(e) => {
                                error!(
                                    "Failed to find penultimate Index to revert to({}.{}): {}",
                                    &request.namespace, &request.identifier, e
                                );

                                continue;
                            }
                        }
                    }
                },
                Err(e) => {
                    debug!("No service request to handle: {e:?}");
                    sleep(Duration::from_secs(defaults::IDLE_SERVICE_WAIT_SECS)).await;
                }
            }
        }
    }
}
