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
use fuel_indexer_lib::utils::{FuelNodeHealthResponse, ServiceStatus};
use fuel_indexer_postgres;
use fuel_indexer_schema::db::{
    graphql::{GraphqlError, GraphqlQueryBuilder},
    models, run_migration,
    tables::Schema,
    IndexerConnectionPool,
};
use fuel_indexer_sqlite;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
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

#[allow(unused_variables)]
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
    let resp = client
        .get(
            format!("{}/health", config.fuel_node.http_url())
                .parse()
                .expect("Failed to parse string into URI"),
        )
        .await
        .expect("Failed to get fuel-client status.");

    let body_bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .expect("Failed to parse response body.");

    let fuel_node_health: FuelNodeHealthResponse =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response.");

    (
        StatusCode::OK,
        Json(json!({
            "fuel_core_status": ServiceStatus::from(fuel_node_health),
            "uptime(seconds)": uptime,
            "database_status": db_status,
        })),
    )
}

async fn authenticate_user(_user_id: &str) -> Option<Result<bool, APIError>> {
    // TODO: Placeholder until actual authentication scheme is in place
    Some(Ok(true))
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetMetadata {
    namespace: String,
    identifier: String,
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

pub async fn asset_upload(
    Extension(schema_manager): Extension<Arc<RwLock<SchemaManager>>>,
    Extension(pool): Extension<IndexerConnectionPool>,
    multipart: Option<Multipart>,
) -> (StatusCode, Json<Value>) {
    if let Some(mut multipart) = multipart {
        let mut items: HashMap<String, Vec<u8>> = HashMap::new();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            items.insert(name.to_string(), data.to_vec());
        }

        // Metadata is a required field
        let metadata: AssetMetadata = serde_json::from_slice(
            items
                .get("metadata")
                .expect("'metadata' field required in multipart upload."),
        )
        .expect("Failed to deserialize metadata.");

        let wasm = items.get("wasm").map(|x| x.to_owned());
        let manifest = items.get("manifest").map(|x| x.to_owned());
        let schema = items.get("schema").map(|x| x.to_owned());

        match pool {
            IndexerConnectionPool::Postgres(p) => {
                let mut conn = p.acquire().await.expect("Failed to get pool connection");
                fuel_indexer_postgres::register_index_assets(
                    &mut conn,
                    &metadata.identifier,
                    &metadata.namespace,
                    wasm,
                    manifest,
                    schema.clone(),
                )
                .await
                .expect("Foobar");
            }
            IndexerConnectionPool::Sqlite(p) => {
                let mut conn = p.acquire().await.expect("Failed to get pool connection");
                fuel_indexer_sqlite::register_index_assets(
                    &mut conn,
                    &metadata.identifier,
                    &metadata.namespace,
                    wasm,
                    manifest,
                    schema.clone(),
                )
                .await
                .expect("Barfoo");
            }
        }

        // TODO: reload new data into service
        // update schema manager
        if let Some(s) = schema {
            schema_manager
                .write()
                .await
                .new_schema(&metadata.namespace, &String::from_utf8_lossy(&s))
                .await
                .expect("Bird");
        }
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
            run_migration(&config.database.to_string()).await;
        }

        let graph_route = Router::new()
            .route("/:name", post(query_graph))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool.clone()));

        let asset_route = Router::new()
            .route("/:name", post(asset_upload))
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
            .nest("/asset", asset_route)
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
