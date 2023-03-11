use crate::uses::{
    authorize_middleware, health_check, metrics, query_graph, register_index_assets,
    revert_indexer, stop_index
};
use async_std::sync::{Arc, RwLock};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Error as AxumError, Router,
};
use fuel_indexer_database::{queries, IndexerConnectionPool, IndexerDatabaseError};
use fuel_indexer_lib::{config::IndexerConfig, utils::ServiceRequest};
use fuel_indexer_schema::db::{
    graphql::GraphqlError, manager::SchemaManager, IndexerSchemaError,
};
use hyper::{Error as HyperError, Method};
use serde_json::json;
use std::{net::SocketAddr, time::Instant};
use thiserror::Error;
use tokio::sync::mpsc::{error::SendError, Sender};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{error, Level};

pub type ApiResult<T> = core::result::Result<T, ApiError>;

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
    #[error("Schema error {0:?}")]
    SchemaError(#[from] IndexerSchemaError),
    #[error("Channel send error: {0:?}")]
    ChannelSendError(#[from] SendError<ServiceRequest>),
    #[error("Axum error: {0:?}")]
    AxumError(#[from] AxumError),
    #[error("Hyper error: {0:?}")]
    HyperError(#[from] HyperError),
}

impl Default for ApiError {
    fn default() -> Self {
        ApiError::Http(HttpError::InternalServer)
    }
}

impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
        match status {
            StatusCode::BAD_REQUEST => ApiError::Http(HttpError::BadRequest),
            StatusCode::UNAUTHORIZED => ApiError::Http(HttpError::Unauthorized),
            _ => ApiError::Http(HttpError::InternalServer),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let generic_err_msg = "Internal server error.".to_string();
        // NOTE: Free to add more specific messaging/handing here as needed
        #[allow(clippy::match_single_binding)]
        let (status, err_msg) = match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, generic_err_msg),
        };

        error!("{:?} - {}", status, err_msg);

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
    pub async fn build(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        tx: Option<Sender<ServiceRequest>>,
    ) -> ApiResult<Router> {
        let sm = SchemaManager::new(pool.clone());
        let schema_manager = Arc::new(RwLock::new(sm));
        let config = config.clone();
        let max_body = config.max_body;
        let start_time = Arc::new(Instant::now());

        if config.graphql_api.run_migrations {
            let mut c = pool.acquire().await?;
            queries::run_migration(&mut c).await?;
        }

        let graph_route = Router::new()
            .route("/:namespace/:identifier", post(query_graph))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool.clone()))
            .layer(RequestBodyLimitLayer::new(max_body));

        let index_routes = Router::new()
            .route("/:namespace/:identifier", post(register_index_assets))
            .route_layer(middleware::from_fn(authorize_middleware))
            .layer(Extension(tx.clone()))
            .layer(Extension(schema_manager))
            .layer(Extension(pool.clone()))
            .route("/:namespace/:identifier", delete(stop_index))
            .route("/:namespace/:identifier", put(revert_index))
            .route_layer(middleware::from_fn(authorize_middleware))
            .layer(Extension(tx))
            .layer(Extension(pool.clone()))
            .layer(RequestBodyLimitLayer::new(max_body));

        let root_routes = Router::new()
            .route("/health", get(health_check))
            .layer(Extension(config))
            .layer(Extension(pool))
            .layer(Extension(start_time))
            .route("/metrics", get(metrics));

        let api_routes = Router::new()
            .nest("/", root_routes)
            .nest("/index", index_routes)
            .nest("/graph", graph_route);

        let app = Router::new()
            .nest("/api", api_routes)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(
                CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::POST])
                    .allow_origin(Any {}),
            );

        Ok(app)
    }

    pub async fn run(config: IndexerConfig, app: Router) -> ApiResult<()> {
        let listen_on: SocketAddr = config.graphql_api.into();

        axum::Server::bind(&listen_on)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }

    pub async fn build_and_run(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        tx: Option<Sender<ServiceRequest>>,
    ) -> ApiResult<()> {
        let listen_on: SocketAddr = config.graphql_api.clone().into();
        let app = GraphQlApi::build(config, pool, tx).await?;

        axum::Server::bind(&listen_on)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
