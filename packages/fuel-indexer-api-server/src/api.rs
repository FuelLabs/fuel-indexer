use crate::{
    auth::AuthenticationMiddleware,
    uses::{
        get_nonce, gql_playground, health_check, metrics, query_graph,
        register_indexer_assets, revert_indexer, stop_indexer, verify_signature,
    },
};
use async_std::sync::{Arc, RwLock};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use fuel_indexer_database::{IndexerConnectionPool, IndexerDatabaseError};
use fuel_indexer_lib::{config::IndexerConfig, utils::ServiceRequest};
use fuel_indexer_schema::db::{
    graphql::GraphqlError, manager::SchemaManager, IndexerSchemaError,
};
use hyper::Method;
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
    #[error("HTTP error: {0:?}")]
    Http(http::Error),
}

impl From<http::Error> for HttpError {
    fn from(err: http::Error) -> Self {
        HttpError::Http(err)
    }
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
}

impl Default for ApiError {
    fn default() -> Self {
        ApiError::Http(HttpError::InternalServer)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let generic_details = "Internal server error.".to_string();
        let (status, details) = match self {
            Self::JsonWebToken(e) => (
                StatusCode::BAD_REQUEST,
                format!("Could not process JWT: {e}"),
            ),
            ApiError::Http(HttpError::Unauthorized) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized.".to_string())
            }
            ApiError::Http(HttpError::NotFound(e)) => {
                (StatusCode::NOT_FOUND, format!("Not found: {e}."))
            }
            ApiError::Sqlx(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}."),
            ),
            ApiError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}."),
            ),
            ApiError::FuelCrypto(e) => {
                (StatusCode::BAD_REQUEST, format!("Crypto error: {e}."))
            }
            ApiError::Graphql(e) => {
                (StatusCode::BAD_REQUEST, format!("GraphQL error: {e}."))
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

impl From<http::Error> for ApiError {
    fn from(err: http::Error) -> Self {
        ApiError::Http(HttpError::from(err))
    }
}

pub struct GraphQlApi;

impl GraphQlApi {
    pub async fn build(
        config: IndexerConfig,
        pool: IndexerConnectionPool,
        tx: Sender<ServiceRequest>,
    ) -> ApiResult<Router> {
        let sm = SchemaManager::new(pool.clone());
        let schema_manager = Arc::new(RwLock::new(sm));
        let max_body_size = config.graphql_api.max_body_size;
        let start_time = Arc::new(Instant::now());

        let graph_route = Router::new()
            .route("/:namespace/:identifier", post(query_graph))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool.clone()))
            .layer(RequestBodyLimitLayer::new(max_body_size));

        let index_routes = Router::new()
            .route("/:namespace/:identifier", post(register_indexer_assets))
            .layer(AuthenticationMiddleware::from(&config))
            .layer(Extension(tx.clone()))
            .layer(Extension(schema_manager.clone()))
            .layer(Extension(pool.clone()))
            .route("/:namespace/:identifier", delete(stop_indexer))
            .route("/:namespace/:identifier", put(revert_indexer))
            .layer(AuthenticationMiddleware::from(&config))
            .layer(Extension(tx))
            .layer(Extension(pool.clone()))
            .layer(RequestBodyLimitLayer::new(max_body_size));

        let root_routes = Router::new()
            .route("/health", get(health_check))
            .layer(Extension(config.clone()))
            .layer(Extension(pool.clone()))
            .layer(Extension(start_time))
            .route("/metrics", get(metrics));

        let auth_routes = Router::new()
            .route("/nonce", get(get_nonce))
            .layer(Extension(pool.clone()))
            .route("/signature", post(verify_signature))
            .layer(Extension(pool.clone()))
            .layer(Extension(config));

        let playground_route = Router::new()
            .route("/:namespace/:identifier", get(gql_playground))
            .layer(Extension(schema_manager))
            .layer(Extension(pool))
            .layer(RequestBodyLimitLayer::new(max_body_size));

        let api_routes = Router::new()
            .nest("/", root_routes)
            .nest("/playground", playground_route)
            .nest("/index", index_routes)
            .nest("/graph", graph_route)
            .nest("/auth", auth_routes);

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
