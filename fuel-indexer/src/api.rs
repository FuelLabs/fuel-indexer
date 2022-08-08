use crate::{IndexerConfig, SchemaManager};
use async_std::sync::{Arc, RwLock};
use axum::{
    extract::{Extension, Json, Path},
    routing::post,
    Router,
};
use fuel_indexer_schema::db::{
    graphql::{GraphqlError, GraphqlQueryBuilder},
    models,
    tables::Schema,
    IndexerConnectionPool,
};
use http::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
enum APIError {
    #[error("Query builder error {0:?}")]
    GraphqlError(#[from] GraphqlError),
    #[error("Serde Error {0:?}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Sqlx Error {0:?}")]
    SqlxError(#[from] sqlx::Error),
}

#[derive(Clone, Debug, Deserialize)]
struct Query {
    query: String,
    #[allow(unused)] // TODO
    params: String,
}

async fn query_graph(
    Path(name): Path<String>,
    Json(query): Json<Query>,
    Extension(pool): Extension<&IndexerConnectionPool>,
    Extension(manager): Extension<Arc<RwLock<SchemaManager>>>,
) -> (StatusCode, Json<Value>) {
    match manager.read().await.load_schema_wasm(&name).await {
        Ok(schema) => match run_query(query, schema, pool).await {
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

pub struct GraphQlApi;

impl GraphQlApi {
    pub async fn run(config: IndexerConfig) {
        let sm = SchemaManager::new(&config.database_config.to_string())
            .await
            .expect("SchemaManager create failed");
        let schema_manager = Arc::new(RwLock::new(sm));
        let config = Arc::new(config.clone());
        let listen_on = config.graphql_api.clone().into();

        let pool = IndexerConnectionPool::connect(&config.database_config.to_string())
            .await
            .expect("Failed to establish connection pool");

        let app = Router::new()
            .route("/graph/:name", post(query_graph))
            .layer(Extension(schema_manager))
            .layer(Extension(pool));

        axum::Server::bind(&listen_on)
            .serve(app.into_make_service())
            .await
            .expect("Service failed to start");
    }
}

async fn run_query(
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
            let row: Value = serde_json::from_str(&ans)?;
            Ok(row)
        }
        Err(e) => {
            error!("Error querying database");
            Err(e.into())
        }
    }
}
