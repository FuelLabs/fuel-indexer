use crate::uses::{
    authorize_middleware, health_check, metrics, query_graph, register_index_assets,
    stop_index,
};
use async_std::sync::{Arc, RwLock};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    middleware::{self},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use fuel_indexer_database::{queries, IndexerConnectionPool, IndexerDatabaseError};
use fuel_indexer_lib::{
    config::{IndexerConfig, MutableConfig},
    utils::ServiceRequest,
};
use fuel_indexer_schema::db::{
    graphql::GraphqlError, manager::SchemaManager, IndexerSchemaError,
};
use serde_json::json;
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
    #[error("Generic error")]
    Generic,
    #[error("Schema error {0:?}")]
    SchemaError(#[from] IndexerSchemaError),
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

pub struct GraphQlApi;

impl GraphQlApi {
    pub async fn run(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        tx: Option<Sender<ServiceRequest>>,
    ) {
        let sm = SchemaManager::new(pool.clone());
        let schema_manager = Arc::new(RwLock::new(sm));
        let config = config.clone();
        let start_time = Arc::new(Instant::now());
        let listen_on = config.graphql_api.derive_socket_addr();

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
            .layer(Extension(tx.clone()))
            .layer(Extension(schema_manager))
            .layer(Extension(pool.clone()));

        let stop_index_route = Router::new()
            .route("/:namespace/:identifier", delete(stop_index))
            .route_layer(middleware::from_fn(authorize_middleware))
            .layer(Extension(tx));

        let health_route = Router::new()
            .route("/health", get(health_check))
            .layer(Extension(config))
            .layer(Extension(pool))
            .layer(Extension(start_time));

        let metrics_route = Router::new().route("/metrics", get(metrics));

        let api_routes = Router::new()
            .nest("/", health_route)
            .nest("/", metrics_route)
            .nest("/index", asset_route)
            .nest("/index", stop_index_route)
            .nest("/graph", graph_route);

        let app = Router::new().nest("/api", api_routes);

        axum::Server::bind(&listen_on)
            .serve(app.into_make_service())
            .await
            .expect("Service failed to start");
    }
}
