use anyhow::Result;
use async_std::sync::{Arc, RwLock};
use async_std::{fs::File, io::ReadExt};
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    routing::post,
    Router,
};
use diesel::prelude::PgConnection as Conn;
use diesel::sql_types::Text;
use diesel::{Connection, QueryableByName, RunQueryDsl};
pub use fuel_indexer_lib::{
    config::{ApiServerArgs, GraphQLConfig, Parser, PostgresConfig},
    defaults,
};
use fuel_indexer_schema::db::{
    graphql::{GraphqlError, GraphqlQueryBuilder},
    run_migration,
    tables::Schema,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tracing::error;

#[derive(QueryableByName)]
pub struct Answer {
    #[sql_type = "Text"]
    row: String,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Diesel error {0:?}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Diesel connection error {0:?}")]
    DieselConnectionError(#[from] diesel::result::ConnectionError),
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
    Extension(database_url): Extension<String>,
    Extension(manager): Extension<Arc<RwLock<SchemaManager>>>,
) -> (StatusCode, Json<Value>) {
    if !manager.read().await.contains_key(&name) {
        if let Ok(Some(schema)) = load_schema_wasm(&database_url, &name).await {
            manager.write().await.insert(name.clone(), schema);
        } else {
            let result = format!("The graph {name} was not found.");
            return (StatusCode::NOT_FOUND, Json(Value::String(result)));
        }
    };

    let guard = manager.read().await;
    let schema = guard.get(&name).unwrap();

    match run_query(query, schema, database_url).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => {
            error!("Query error {e:?}");
            let res = Json(Value::String("Internal Server Error".into()));
            (StatusCode::INTERNAL_SERVER_ERROR, res)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiServerConfig {
    pub graphql_api: GraphQLConfig,
    pub postgres: PostgresConfig,
}

impl ApiServerConfig {
    pub async fn from_file(path: &std::path::Path) -> Result<Self> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let config = serde_yaml::from_str(&contents)?;

        Ok(config)
    }

    pub fn from_opts(args: ApiServerArgs) -> ApiServerConfig {
        ApiServerConfig {
            postgres: PostgresConfig {
                user: args
                    .postgres_user
                    .unwrap_or_else(|| defaults::POSTGRES_USER.into()),
                password: args.postgres_password,
                host: args
                    .postgres_host
                    .unwrap_or_else(|| defaults::POSTGRES_HOST.into()),
                port: args
                    .postgres_port
                    .unwrap_or_else(|| defaults::POSTGRES_PORT.into()),
                database: args.postgres_database,
            },
            graphql_api: GraphQLConfig {
                host: args
                    .graphql_api_host
                    .unwrap_or_else(|| defaults::GRAPHQL_API_HOST.into()),
                port: args
                    .graphql_api_port
                    .unwrap_or_else(|| defaults::GRAPHQL_API_PORT.into()),
            },
        }
    }
}

pub struct GraphQLApi {
    pg_config: PostgresConfig,
    graphql_config: GraphQLConfig,
}

impl GraphQLApi {
    pub fn new(pg_config: PostgresConfig, graphql_config: GraphQLConfig) -> GraphQLApi {
        GraphQLApi {
            pg_config,
            graphql_config,
        }
    }

    pub async fn run(self) {
        let sm = SchemaManager::new();
        let schema_manager = Arc::new(RwLock::new(sm));

        run_migration(&self.pg_config.to_string());

        let app = Router::new()
            .route("/graph/:name", post(query_graph))
            .layer(Extension(self.pg_config.to_string()))
            .layer(Extension(schema_manager));

        axum::Server::bind(&self.graphql_config.into())
            .serve(app.into_make_service())
            .await
            .expect("Service failed to start");
    }
}

pub async fn load_schema_wasm(database_url: &str, name: &str) -> Result<Option<Schema>, ApiError> {
    let conn = Conn::establish(database_url)?;
    Ok(Some(Schema::load_from_db(&conn, name)?))
}

pub async fn run_query(
    query: Query,
    schema: &Schema,
    database_url: String,
) -> Result<Value, ApiError> {
    let conn = Conn::establish(&database_url)?;
    let builder = GraphqlQueryBuilder::new(schema, &query.query)?;
    let query = builder.build()?;

    let queries = query.as_sql(true).join(";\n");

    match diesel::sql_query(queries).get_result::<Answer>(&conn) {
        Ok(ans) => {
            let row: Value = serde_json::from_str(&ans.row)?;
            Ok(row)
        }
        Err(diesel::result::Error::NotFound) => Ok(Value::Null),
        Err(e) => {
            error!("Error querying database");
            Err(e.into())
        }
    }
}
