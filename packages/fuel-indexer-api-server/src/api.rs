use crate::{
    middleware::AuthenticationMiddleware,
    uses::{
        get_nonce, gql_playground, health_check, indexer_status, query_graph,
        register_indexer_assets, remove_indexer, verify_signature,
    },
};

#[cfg(feature = "metrics")]
use crate::middleware::MetricsMiddleware;

use async_std::sync::{Arc, RwLock};
use axum::{
    error_handling::HandleErrorLayer,
    extract::{Extension, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use fuel_indexer_database::{IndexerConnectionPool, IndexerDatabaseError};
use fuel_indexer_graphql::graphql::GraphqlError;
use fuel_indexer_lib::{config::IndexerConfig, defaults, utils::ServiceRequest};
use fuel_indexer_schema::db::{manager::SchemaManager, IndexerSchemaDbError};
use hyper::Method;
use serde_json::json;
use std::{
    net::SocketAddr,
    str::FromStr,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::mpsc::{error::SendError, Sender};
use tower::ServiceBuilder;
use tower::{buffer::BufferLayer, limit::RateLimitLayer};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{error, Level};

/// Result type returned by web API operations.
pub type ApiResult<T> = core::result::Result<T, ApiError>;

/// Size of the buffer for reqeusts being passed to the `RateLimitLayer`.
const BUFFER_SIZE: usize = 1024;

/// Error type returned by HTTP operations.
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Bad request.")]
    BadRequest,
    #[error("Conflict. {0:#?}")]
    Conflict(String),
    #[error("Unauthorized request.")]
    Unauthorized,
    #[error("Not not found. {0:#?}")]
    NotFound(String),
    #[error("Error.")]
    InternalServer,
    #[error("HTTP error: {0:?}")]
    Http(http::Error),
}

impl From<http::Error> for HttpError {
    fn from(err: http::Error) -> Self {
        HttpError::Http(err)
    }
}

/// Error type returned by web API operations.
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
    SchemaError(#[from] IndexerSchemaDbError),
    #[error("Channel send error: {0:?}")]
    ChannelSend(#[from] SendError<ServiceRequest>),
    #[error("Axum error: {0:?}")]
    Axum(#[from] axum::Error),
    #[error("Hyper error: {0:?}")]
    HyperError(#[from] hyper::Error),
    #[error("FuelCrypto error: {0:?}")]
    FuelCrypto(#[from] fuel_crypto::Error),
    #[error("JsonWebToken: {0:?}")]
    JsonWebToken(#[from] jsonwebtoken::errors::Error),
    #[error("HexError: {0:?}")]
    HexError(#[from] hex::FromHexError),
    #[error("BoxError: {0:?}")]
    BoxError(#[from] axum::BoxError),
}

impl Default for ApiError {
    fn default() -> Self {
        ApiError::Http(HttpError::InternalServer)
    }
}

impl From<http::Error> for ApiError {
    fn from(err: http::Error) -> Self {
        ApiError::Http(HttpError::from(err))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let generic_details = "Internal server error.".to_string();

        error!("ApiError: {self:?}");

        let (status, details) = match self {
            Self::JsonWebToken(e) => (
                StatusCode::BAD_REQUEST,
                format!("Could not process JWT: {e}"),
            ),
            Self::Http(HttpError::Conflict(e)) => {
                (StatusCode::CONFLICT, format!("Conflict: {e}"))
            }
            Self::Http(HttpError::Unauthorized) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized.".to_string())
            }
            Self::Http(HttpError::NotFound(e)) => {
                (StatusCode::NOT_FOUND, format!("Not found: {e}."))
            }
            Self::Sqlx(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}."),
            ),
            Self::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}."),
            ),
            Self::FuelCrypto(e) => {
                (StatusCode::BAD_REQUEST, format!("Crypto error: {e}."))
            }
            Self::Graphql(e) => (StatusCode::BAD_REQUEST, format!("GraphQL error: {e}.")),
            Self::SchemaError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Schema error: {e}."),
            ),
            ApiError::BoxError(e) => {
                error!("Generic BoxError: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e}"))
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, generic_details),
        };

        error!("{status:?} - {details}");

        (
            status,
            Json(json!({
                "success": "false",
                "details": details,
            })),
        )
            .into_response()
    }
}

/// GraphQL API server.
pub struct GraphQlApi;

impl GraphQlApi {
    /// Build an `axum` application with all routes.
    pub async fn build(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        tx: Sender<ServiceRequest>,
    ) -> ApiResult<Router> {
        let sm = SchemaManager::new(pool.clone());
        let schema_manager = Arc::new(RwLock::new(sm));
        let max_body_size = config.graphql_api.max_body_size;
        let start_time = Arc::new(Instant::now());
        let log_level = Level::from_str(config.log_level.as_ref()).unwrap();

        let mut graph_routes = Router::new()
            .route("/:namespace/:identifier", post(query_graph))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool.clone()))
            .layer(RequestBodyLimitLayer::new(max_body_size));

        if config.rate_limit.enabled {
            graph_routes = graph_routes.layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(|e: axum::BoxError| async move {
                        ApiError::from(e).into_response()
                    }))
                    .layer(BufferLayer::new(BUFFER_SIZE))
                    .layer(RateLimitLayer::new(
                        config
                            .rate_limit
                            .request_count
                            .unwrap_or(defaults::RATE_LIMIT_REQUEST_COUNT),
                        Duration::from_secs(
                            config
                                .rate_limit
                                .window_size
                                .unwrap_or(defaults::RATE_LIMIT_WINDOW_SIZE),
                        ),
                    )),
            );
        }

        #[cfg(feature = "metrics")]
        let graph_routes = graph_routes.layer(MetricsMiddleware::default());

        let indexer_routes = Router::new()
            .route("/:namespace/:identifier", post(register_indexer_assets))
            .layer(AuthenticationMiddleware::from(&config))
            .layer(Extension(tx.clone()))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool.clone()))
            .layer(Extension(config.clone()))
            .route("/:namespace/:identifier", delete(remove_indexer))
            .layer(AuthenticationMiddleware::from(&config))
            .layer(Extension(tx))
            .layer(Extension(pool.clone()))
            .layer(Extension(config.clone()))
            .layer(RequestBodyLimitLayer::new(max_body_size));

        #[cfg(feature = "metrics")]
        let indexer_routes = indexer_routes.layer(MetricsMiddleware::default());

        let root_routes = Router::new()
            .route("/status", get(indexer_status))
            .layer(Extension(pool.clone()))
            .layer(AuthenticationMiddleware::from(&config))
            .layer(Extension(config.clone()))
            .route("/health", get(health_check))
            .layer(Extension(config.clone()))
            .layer(Extension(pool.clone()))
            .layer(Extension(start_time));

        #[cfg(feature = "metrics")]
        let root_routes = root_routes
            .route("/metrics", get(crate::uses::get_metrics))
            .layer(MetricsMiddleware::default());

        let auth_routes = Router::new()
            .route("/nonce", get(get_nonce))
            .layer(Extension(pool.clone()))
            .route("/signature", post(verify_signature))
            .layer(Extension(pool.clone()))
            .layer(Extension(config));

        #[cfg(feature = "metrics")]
        let auth_routes = auth_routes.layer(MetricsMiddleware::default());

        let playground_route = Router::new()
            .route("/:namespace/:identifier", get(gql_playground))
            .layer(Extension(schema_manager))
            .layer(Extension(pool))
            .layer(RequestBodyLimitLayer::new(max_body_size));

        #[cfg(feature = "metrics")]
        let playground_route = playground_route.layer(MetricsMiddleware::default());

        let api_routes = Router::new()
            .nest("/", root_routes)
            .nest("/playground", playground_route)
            .nest("/index", indexer_routes)
            .nest("/graph", graph_routes)
            .nest("/auth", auth_routes);

        let app = Router::new()
            .nest("/api", api_routes)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(log_level))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(log_level)
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

    /// Start the GraphQL API server.
    pub async fn run(config: IndexerConfig, app: Router) -> ApiResult<()> {
        let listen_on: SocketAddr = config.graphql_api.into();

        axum::Server::bind(&listen_on)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }

    /// Build a GraphQL API web server as an `axum` application and start the server.
    ///
    /// This function is a combination of `GraphQLApi::build` and `GraphQLApi::start`
    pub async fn build_and_run(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        tx: Sender<ServiceRequest>,
    ) -> ApiResult<()> {
        let listen_on: SocketAddr = config.graphql_api.clone().into();
        let app = GraphQlApi::build(config, pool, tx).await?;

        axum::Server::bind(&listen_on)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
