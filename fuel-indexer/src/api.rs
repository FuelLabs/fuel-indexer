use crate::{
    config::{AdjustableConfig, IndexerConfig},
    Manifest, SchemaManager,
};
use anyhow::Result;
use async_std::{
    fs::File,
    io::WriteExt,
    sync::{Arc, RwLock},
};
use axum::{
    extract::{multipart::Multipart, Extension, Json, Path},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::post,
    Router,
};
use fuel_indexer_schema::db::{
    graphql::{GraphqlError, GraphqlQueryBuilder},
    models, run_migration,
    tables::Schema,
    IndexerConnectionPool,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum APIError {
    #[error("Query builder error {0:?}")]
    Graphql(#[from] GraphqlError),
    #[error("Serde Error {0:?}")]
    Serde(#[from] serde_json::Error),
    #[error("Sqlx Error {0:?}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Query {
    query: String,
    #[allow(unused)] // TODO
    params: String,
}

pub async fn query_graph(
    Path(name): Path<String>,
    Json(query): Json<Query>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(manager): Extension<Arc<RwLock<SchemaManager>>>,
) -> (StatusCode, Json<Value>) {
    match manager.read().await.load_schema_wasm(&name).await {
        Ok(schema) => match run_query(query, schema, &pool).await {
            Ok(response) => (StatusCode::OK, Json(response)),
            Err(e) => {
                error!("Query error {e:?}");
                let res = Json(Value::String("Internal Server Error".into()));
                (StatusCode::INTERNAL_SERVER_ERROR, res)
            }
        },
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(Value::String(format!(
                "The graph {} was not found ({:?})",
                name, e
            ))),
        ),
    }
}

async fn authenticate_user(_user_id: &str) -> Option<Result<bool, APIError>> {
    // TODO: Placeholder until actual authentication scheme is in place
    Some(Ok(true))
}

async fn authorize_middleware<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    let auth_header = if let Some(auth_header) = header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(current_user) = authenticate_user(auth_header).await {
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AssetMetadata {
    index_name: String,
}

#[derive(Default, Debug)]
struct AssetBundle {
    pub wasm_bytes: Option<Vec<u8>>,
    pub schema_bytes: Option<Vec<u8>>,
    pub manifest: Manifest,
    pub metadata: AssetMetadata,
}

impl AssetMetadata {
    pub fn filename_for(&self, asset: Asset) -> String {
        format!("{}{}", self.index_name, asset.file_extension())
    }
}

impl AssetBundle {
    /// Construct an `AssetBundle` using `items` parsed from the `Multipart`
    pub async fn new(items: HashMap<Asset, Vec<u8>>, config: Arc<IndexerConfig>) -> Result<Self> {
        // Metadata should always be sent so that we know what index we're working with
        let metadata_bytes = items.get(&Asset::Metadata).unwrap();
        let metadata: AssetMetadata =
            serde_json::from_slice(metadata_bytes).expect("Invalid metadata.");

        // The manifest _must_ exist. Either it has to be sent as a part of the upload, or it
        // has to already exist in the asset directory
        let manifest_bytes = match items.get(&Asset::Manifest) {
            Some(v) => v.to_owned(),
            None => {
                let path = config
                    .fuel_indexer_manifest_dir()
                    .join(metadata.index_name.clone());
                Manifest::from_file_as_bytes(&path).expect("Manifest not found.")
            }
        };

        let manifest_content = std::str::from_utf8(&manifest_bytes[..]).unwrap();

        Ok(Self {
            wasm_bytes: items.get(&Asset::Wasm).map(|v| v.to_owned()),
            schema_bytes: items.get(&Asset::Schema).map(|v| v.to_owned()),
            manifest: serde_yaml::from_str(manifest_content).unwrap(),
            metadata,
        })
    }

    /// Deconstruct a `Multipart` upload into different `Asset` pieces, and build an `AssetBundle`
    pub async fn into_map(mut multipart: Multipart, config: Arc<IndexerConfig>) -> Result<Self> {
        let mut items: HashMap<Asset, Vec<u8>> = HashMap::new();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            match name.into() {
                Asset::Wasm => {
                    items.insert(Asset::Wasm, data.to_vec());
                }
                Asset::Schema => {
                    items.insert(Asset::Schema, data.to_vec());
                }
                Asset::Manifest => {
                    items.insert(Asset::Manifest, data.to_vec());
                }
                Asset::Metadata => {
                    items.insert(Asset::Metadata, data.to_vec());
                }
            }
        }

        let bundle = AssetBundle::new(items, config).await?;

        Ok(bundle)
    }
}

pub struct WasmLock;
pub struct SchemaLock;
pub struct ManifestLock;

#[derive(Eq, Hash, PartialEq)]
pub enum Asset {
    Wasm,
    Schema,
    Manifest,
    Metadata,
}

impl Asset {
    pub fn file_extension(&self) -> String {
        match self {
            Asset::Wasm => ".wasm".to_string(),
            Asset::Schema => ".graphql".to_string(),
            Asset::Manifest => ".yaml".to_string(),
            Asset::Metadata => ".json".to_string(),
        }
    }
}

impl From<String> for Asset {
    fn from(s: String) -> Self {
        match s.as_str() {
            "wasm" => Self::Wasm,
            "schema" => Self::Schema,
            "manifest" => Self::Manifest,
            "metadata" => Self::Metadata,
            _ => panic!("Unrecognized mutlipart field."),
        }
    }
}

impl std::string::ToString for Asset {
    fn to_string(&self) -> String {
        match self {
            Asset::Wasm => "wasm".to_string(),
            Asset::Schema => "schema".to_string(),
            Asset::Manifest => "manifest".to_string(),
            Asset::Metadata => "metadata".to_string(),
        }
    }
}

pub async fn asset_upload(
    Path(name): Path<String>,
    Extension(schema_manager): Extension<Arc<RwLock<SchemaManager>>>,
    Extension(wasm_lock): Extension<Arc<RwLock<WasmLock>>>,
    Extension(schema_lock): Extension<Arc<RwLock<SchemaLock>>>,
    Extension(manifest_lock): Extension<Arc<RwLock<ManifestLock>>>,
    Extension(config): Extension<Arc<IndexerConfig>>,
    multipart: Option<Multipart>,
) -> (StatusCode, Json<Value>) {
    let schema = schema_manager.read().await.load_schema_wasm(&name).await;

    if schema.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(Value::String(format!(
                "The graph {} was not found ({:?})",
                name, schema
            ))),
        );
    }

    if let Some(multipart) = multipart {
        let mut bundle = AssetBundle::into_map(multipart, config.clone())
            .await
            .unwrap();

        // Update the manifest with WASM location and write the WASM
        if let Some(bytes) = bundle.wasm_bytes {
            wasm_lock.write().await;
            bundle.manifest.wasm_module = Some(
                config
                    .fuel_indexer_wasm_dir()
                    .join(bundle.metadata.filename_for(Asset::Wasm)),
            );
            let mut file = File::create(bundle.manifest.wasm_module.as_ref().unwrap())
                .await
                .unwrap();
            let _n = file.write(&bytes).await.unwrap();
        }

        // Update the manifest with schema location and write the schema
        if let Some(bytes) = bundle.schema_bytes {
            schema_lock.write().await;
            bundle.manifest.graphql_schema = config
                .fuel_indexer_schema_dir()
                .join(bundle.metadata.filename_for(Asset::Schema));
            let mut file = File::create(&bundle.manifest.graphql_schema).await.unwrap();
            let _n = file.write(&bytes).await.unwrap();
        }

        // Write the updated manifest
        manifest_lock.write().await;
        let mut file = File::create(
            config
                .fuel_indexer_manifest_dir()
                .join(bundle.metadata.filename_for(Asset::Manifest)),
        )
        .await
        .unwrap();
        let _n = file
            .write(&bundle.manifest.to_bytes().unwrap())
            .await
            .unwrap();

        // TODO: Signal that the app needs a restart
    }

    (StatusCode::OK, Json(Value::String("Success".to_string())))
}

pub struct GraphQlApi;

impl GraphQlApi {
    pub async fn run(config: IndexerConfig) {
        let sm = SchemaManager::new(&config.database.to_string())
            .await
            .expect("SchemaManager create failed");
        let schema_manager = Arc::new(RwLock::new(sm));
        let config = Arc::new(config.clone());
        let listen_on = config.graphql_api.derive_socket_addr().unwrap();

        let wasm_lock = Arc::new(RwLock::new(WasmLock {}));
        let schema_lock = Arc::new(RwLock::new(SchemaLock {}));
        let manifest_lock = Arc::new(RwLock::new(ManifestLock {}));

        let pool = IndexerConnectionPool::connect(&config.database.to_string())
            .await
            .expect("Failed to establish connection pool");

        if config.graphql_api.run_migrations {
            run_migration(&config.database.to_string()).await;
        }

        let graph_routes = Router::new()
            .route("/:name", post(query_graph))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool));

        let asset_routes = Router::new()
            .route("/:name", post(asset_upload))
            .route_layer(middleware::from_fn(authorize_middleware))
            .layer(Extension(schema_manager))
            .layer(Extension(wasm_lock))
            .layer(Extension(schema_lock))
            .layer(Extension(manifest_lock))
            .layer(Extension(config));

        let api_routes = Router::new()
            .nest("/graph", graph_routes)
            .nest("/asset", asset_routes);

        let app = Router::new().nest("/api", api_routes);

        axum::Server::bind(&listen_on)
            .serve(app.into_make_service())
            .await
            .expect("Service failed to start");
    }
}

pub async fn run_query(
    query: Query,
    schema: Schema,
    pool: &IndexerConnectionPool,
) -> Result<Value, APIError> {
    let builder = GraphqlQueryBuilder::new(&schema, &query.query)?;
    let query = builder.build()?;

    let queries = query.as_sql(true).join(";\n");

    let mut conn = pool.acquire().await?;

    match models::run_query(&mut conn, queries).await {
        Ok(ans) => {
            let row: Value = serde_json::from_value(ans)?;
            Ok(row)
        }
        Err(e) => {
            error!("Error querying database");
            Err(e.into())
        }
    }
}
