use crate::{
    config::{IndexerConfig, MutableConfig},
    SchemaManager,
};
use anyhow::Result;
use async_std::sync::{Arc, RwLock};
use axum::{
    extract::{multipart::Multipart, Extension, Json, Path},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_database_types::IndexAsset;
use fuel_indexer_lib::utils::{FuelNodeHealthResponse, ServiceStatus};
use fuel_indexer_schema::db::{
    graphql::{GraphqlError, GraphqlQueryBuilder},
    tables::Schema,
};
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Instant;
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

pub async fn health_check(
    Extension(config): Extension<IndexerConfig>,
    Extension(pool): Extension<IndexerConnectionPool>,
    Extension(start_time): Extension<Arc<Instant>>,
) -> (StatusCode, Json<Value>) {
    // Get database status
    let db_status = pool.is_connected().await.unwrap_or(ServiceStatus::NotOk);

    let uptime = start_time.elapsed().as_secs().to_string();

    // Get fuel-core status
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let uri = match format!("{}/health", config.fuel_node.http_url()).parse() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to parse string into URI: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Value::String("Failed to parse string into URI".into())),
            );
        }
    };

    let resp = match client.get(uri).await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to get fuel-client status: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Value::String("Failed to get fuel-client status".into())),
            );
        }
    };

    let body_bytes = match hyper::body::to_bytes(resp.into_body()).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to parse response body: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Value::String("Failed to parse response body".into())),
            );
        }
    };

    let fuel_node_health: FuelNodeHealthResponse = match serde_json::from_slice(&body_bytes) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse response: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Value::String("Failed to parse response".into())),
            );
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "fuel_core_status": ServiceStatus::from(fuel_node_health),
            "uptime(seconds)": uptime,
            "database_status": db_status,
        })),
    )
}

async fn authenticate_user(_signature: &str) -> Option<Result<bool, APIError>> {
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

pub async fn register_index_assets(
    Path((namespace, identifier)): Path<(String, String)>,
    Extension(schema_manager): Extension<Arc<RwLock<SchemaManager>>>,
    Extension(pool): Extension<IndexerConnectionPool>,
    multipart: Option<Multipart>,
) -> (StatusCode, Json<Value>) {
    if let Some(mut multipart) = multipart {
        let mut conn = match pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Value::String("Failed to get database connection".into())),
                );
            }
        };

        match queries::start_transaction(&mut conn).await {
            Ok(_) => {}
            Err(e) => {
                error!("Could not start database transaction: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Value::String("Could not start database transaction".into())),
                );
            }
        };

        let mut assets: Vec<IndexAsset> = Vec::new();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = match field.name() {
                Some(n) => n.to_string(),
                None => {
                    error!("Failed to read multipart field");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(Value::String("Failed to read multipart field".into())),
                    );
                }
            };

            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to read multipart body: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(Value::String("Failed to read multipart body".into())),
                    );
                }
            };

            let asset = match name.as_str() {
                "wasm" | "manifest" => {
                    match queries::register_index_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        name.into(),
                    )
                    .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            error!(
                                "Failed to register index asset of type wasm or manifest: {}",
                                e
                            );
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(Value::String(
                                    "Failed to register index asset of type wasm or manifest"
                                        .into(),
                                )),
                            );
                        }
                    }
                }

                "schema" => {
                    let asset = match queries::register_index_asset(
                        &mut conn,
                        &namespace,
                        &identifier,
                        data.to_vec(),
                        name.into(),
                    )
                    .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            error!("Failed to register index asset of type schema: {}", e);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(Value::String(
                                    "Failed to register index asset of type schema".into(),
                                )),
                            );
                        }
                    };

                    if (schema_manager
                        .write()
                        .await
                        .new_schema(&namespace, &String::from_utf8_lossy(&data))
                        .await)
                        .is_err()
                    {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(Value::String(
                                "Failed to generate new schema for asset".into(),
                            )),
                        );
                    }

                    asset
                }
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "success": "false",
                            "details": "Accepted fields are wasm, schema, and manifest.",
                        })),
                    )
                }
            };

            assets.push(asset);
        }

        if (queries::commit_transaction(&mut conn).await).is_err()
            && (queries::revert_transaction(&mut conn).await).is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Value::String(
                    "Could not revert database transaction".into(),
                )),
            );
        };

        return (
            StatusCode::OK,
            Json(json!({
                "success": "true",
                "assets": assets,
            })),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": "false",
            "details": "No multipart data provided.",
        })),
    )
}

pub struct GraphQlApi;

impl GraphQlApi {
    pub async fn run(config: IndexerConfig) {
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
) -> Result<Value, APIError> {
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
