use async_std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
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
pub enum ApiError {
    #[error("Query builder error {0:?}")]
    GraphqlError(#[from] GraphqlError),
    #[error("Malformed query")]
    MalformedQuery,
    #[error("Unexpected DB type {0:?}")]
    UnexpectedDBType(String),
    #[error("Serde Error {0:?}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Graph Not Found.")]
    GraphNotFound,
    #[error("Sqlx Error.")]
    SqlxError(#[from] sqlx::Error),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Query {
    query: String,
    #[allow(unused)] // TODO
    params: String,
}

type SchemaManager = HashMap<String, Schema>;

async fn query_graph(
    Path(name): Path<String>,
    Json(query): Json<Query>,
    Extension(pool): Extension<&IndexerConnectionPool>,
    Extension(manager): Extension<Arc<RwLock<SchemaManager>>>,
) -> (StatusCode, Json<Value>) {
    if !manager.read().await.contains_key(&name) {
        if let Ok(Some(schema)) = load_schema_wasm(&pool, &name).await {
            manager.write().await.insert(name.clone(), schema);
        } else {
            let result = format!("The graph {name} was not found.");
            return (StatusCode::NOT_FOUND, Json(Value::String(result)));
        }
    };

    let guard = manager.read().await;
    let schema = guard.get(&name).unwrap();

    match run_query(query, schema, pool).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => {
            error!("Query error {e:?}");
            let res = Json(Value::String("Internal Server Error".into()));
            (StatusCode::INTERNAL_SERVER_ERROR, res)
        }
    }
}

pub struct GraphQlApi {
    database_url: String,
    listen_address: SocketAddr,
}

impl GraphQlApi {
    pub fn new(database_url: String, listen_address: SocketAddr) -> GraphQlApi {
        GraphQlApi {
            database_url,
            listen_address,
        }
    }

    pub async fn run(self) {
        let sm = SchemaManager::new();
        let schema_manager = Arc::new(RwLock::new(sm));

        run_migration(&self.database_url).await;

        let pool = IndexerConnectionPool::connect(&self.database_url).await;
        let app = Router::new()
            .route("/graph/:name", post(query_graph))
            .layer(Extension(pool))
            .layer(Extension(schema_manager));

        axum::Server::bind(&self.listen_address)
            .serve(app.into_make_service())
            .await
            .expect("Service failed to start");
    }
}

pub async fn load_schema_wasm(
    pool: &IndexerConnectionPool,
    name: &str,
) -> Result<Option<Schema>, ApiError> {
    Ok(Some(Schema::load_from_db(pool, name).await?))
}

pub async fn run_query(
    query: Query,
    schema: &Schema,
    pool: &IndexerConnectionPool,
) -> Result<Value, ApiError> {
    let mut conn = pool.acquire().await?;
    let builder = GraphqlQueryBuilder::new(schema, &query.query)?;
    let query = builder.build()?;

    let queries = query.as_sql(true).join(";\n");

    match models::run_query(&mut conn, queries).await {
        Ok(ans) => {
            let row: Value = serde_json::from_str(&ans)?;
            Ok(row)
        }
        Err(e) => {
            error!("Error querying database");
            Err(e.into())
        }
    }
}
