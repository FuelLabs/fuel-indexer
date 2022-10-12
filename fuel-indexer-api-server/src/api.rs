use anyhow::Result;
use async_std::sync::{Arc, RwLock};
use axum::{
    extract::{multipart::Multipart, Extension, Json, Path},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use fuel_indexer_database::{queries, IndexerConnectionPool, IndexerDatabaseError};
use fuel_indexer_database_types::{IndexAsset, IndexAssetType};
use fuel_indexer_lib::config::{IndexerConfig, MutableConfig};
use fuel_indexer_lib::utils::{
    AssetReloadRequest, FuelNodeHealthResponse, ServiceStatus,
};
use fuel_indexer_schema::db::{
    graphql::{GraphqlError, GraphqlQueryBuilder},
    tables::{Schema, SchemaManager},
};
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Instant;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Bad request.")]
    BadRequest,
    #[error("Unauthorized request.")]
    Unauthorized,
    #[error("Not not found. {0:#?}")]
    NotFound(String),
    #[error("Error.")]
    InternalServer,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Query builder error {0:?}")]
    Graphql(#[from] GraphqlError),
    #[error("Serialization error {0:?}")]
    Serde(#[from] serde_json::Error),
    #[error("Database error {0:?}")]
    Database(#[from] IndexerDatabaseError),
    #[error("Sqlx error {0:?}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Http error {0:?}")]
    Http(#[from] HttpError),
}

impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
        match status {
            // TODO: Finish as needed`
            StatusCode::BAD_REQUEST => ApiError::Http(HttpError::BadRequest),
            StatusCode::UNAUTHORIZED => ApiError::Http(HttpError::Unauthorized),
            _ => ApiError::Http(HttpError::InternalServer),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let generic_err_msg = "Inernal server error.".to_string();
        let (status, err_msg) = match self {
            ApiError::Graphql(err) => {
                error!("ApiError::Graphql: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, generic_err_msg)
            }
            ApiError::Serde(err) => {
                error!("ApiError::Serde: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, generic_err_msg)
            }
            ApiError::Database(err) => {
                error!("ApiError::Database: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, generic_err_msg)
            }
            ApiError::Sqlx(err) => {
                error!("ApiError::Sqlx: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, generic_err_msg)
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, generic_err_msg),
        };

        (
            status,
            Json(json!({
                "success": "false",
                "details": err_msg,
            })),
        )
            .into_response()
    }
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
) -> impl IntoResponse {
    match manager.read().await.load_schema_wasm(&name).await {
        Ok(schema) => match run_query(query, schema, &pool).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => Err(e),
        },
        Err(_e) => Err(ApiError::Http(HttpError::NotFound(format!(
            "The graph '{}' was not found.",
            name
        )))),
    }
}

pub async fn get_fuel_status(config: &IndexerConfig) -> ServiceStatus {
    let url = format!("{}/health", config.fuel_node.http_url())
        .parse()
        .expect("Failed to parse fuel /health url.");

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    match client.get(url).await {
        Ok(r) => {
            let body_bytes = hyper::body::to_bytes(r.into_body())
                .await
                .unwrap_or_default();

            let fuel_node_health: FuelNodeHealthResponse =
                serde_json::from_slice(&body_bytes).unwrap_or_default();

            ServiceStatus::from(fuel_node_health)
        }
        Err(e) => {
            error!("Failed to fetch fuel /health status: {}.", e);
            ServiceStatus::NotOk
        }
    }
}

pub async fn health_check(
    Extension(config): Extension<IndexerConfig>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(start_time): Extension<Arc<Instant>>,
) -> impl IntoResponse {
    let db_status = pool.is_connected().await.unwrap_or(ServiceStatus::NotOk);
    let uptime = start_time.elapsed().as_secs().to_string();
    let fuel_core_status = get_fuel_status(&config).await;

    Ok::<axum::Json<Value>, ApiError>(Json(json!({
        "fuel_core_status": fuel_core_status,
        "uptime(seconds)": uptime,
        "database_status": db_status,
    })))
}

async fn authenticate_user(_signature: &str) -> Option<Result<bool, ApiError>> {
    // TODO: Placeholder until actual authentication scheme is in place
    Some(Ok(true))
}

async fn authorize_middleware<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
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

pub async fn register_index_assets(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(tx): Extension<Option<Sender<AssetReloadRequest>>>,
    Extension(schema_manager): Extension<Arc<RwLock<SchemaManager>>>,
    Extension(pool): Extension<IndexerConnectionPool>,
    multipart: Option<Multipart>,
) -> impl IntoResponse {
    if let Some(mut multipart) = multipart {
        let mut conn = match pool.acquire().await {
            Ok(conn) => conn,
            Err(e) => {
                return Err::<axum::Json<serde_json::Value>, ApiError>(e.into());
            }
        };

        if let Err(e) = queries::start_transaction(&mut conn).await {
            return Err(e.into());
        }

        let mut assets: Vec<IndexAsset> = Vec::new();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap_or("").to_string();
            let data = field.bytes().await.unwrap_or_default();

            let asset: IndexAsset = match name.clone().into() {
                IndexAssetType::Wasm | IndexAssetType::Manifest => {
                    match queries::register_index_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        name.into(),
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                IndexAssetType::Schema => {
                    match queries::register_index_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        IndexAssetType::Schema,
                    )
                    .await
                    {
                        Ok(result) => {
                            if let Err(e) = schema_manager
                                .write()
                                .await
                                .new_schema(&namespace, &String::from_utf8_lossy(&data))
                                .await
                            {
                                return Err(e.into());
                            }
                            result
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
            };

            assets.push(asset);
        }

        if let Err(e) = queries::commit_transaction(&mut conn).await {
            if let Err(e) = queries::revert_transaction(&mut conn).await {
                return Err(e.into());
            };
            return Err(e.into());
        };

        if let Some(tx) = tx {
            tx.send(AssetReloadRequest {
                namespace,
                identifier,
            })
            .await
            .unwrap();
        }

        return Ok(Json(json!({
            "success": "true",
            "assets": assets,
        })));
    }

    Err(StatusCode::BAD_REQUEST.into())
}

pub struct GraphQlApi;

impl GraphQlApi {
    pub async fn run(config: IndexerConfig, tx: Option<Sender<AssetReloadRequest>>) {
        let sm = SchemaManager::new(&config.database.to_string())
            .await
            .expect("SchemaManager create failed");
        let schema_manager = Arc::new(RwLock::new(sm));
        let config = config.clone();
        let start_time = Arc::new(Instant::now());
        let listen_on = config
            .graphql_api
            .derive_socket_addr()
            .expect("Failed to derive socket address");

        let pool = IndexerConnectionPool::connect(&config.database.to_string())
            .await
            .expect("Failed to establish connection pool");

        if config.graphql_api.run_migrations.is_some() {
            queries::run_migration(&config.database.to_string()).await;
        }

        let graph_route = Router::new()
            .route("/:namespace", post(query_graph))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool.clone()));

        let asset_route = Router::new()
            .route("/:namespace/:identifier", post(register_index_assets))
            .route_layer(middleware::from_fn(authorize_middleware))
            .layer(Extension(tx))
            .layer(Extension(schema_manager))
            .layer(Extension(pool.clone()));

        let health_route = Router::new()
            .route("/health", get(health_check))
            .layer(Extension(config))
            .layer(Extension(pool))
            .layer(Extension(start_time));

        let api_routes = Router::new()
            .nest("/", health_route)
            .nest("/index", asset_route)
            .nest("/graph", graph_route);

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
) -> Result<Value, ApiError> {
    let builder = GraphqlQueryBuilder::new(&schema, &query.query)?;
    let query = builder.build()?;

    let queries = query.as_sql(true).join(";\n");

    let mut conn = pool.acquire().await?;

    match queries::run_query(&mut conn, queries).await {
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
