pub use crate::{
    config::{
        auth::{AuthenticationConfig, AuthenticationStrategy},
        client::FuelClientConfig,
        database::DatabaseConfig,
        limit::RateLimitConfig,
        web::WebApiConfig,
    },
    defaults,
};
pub use clap::{Args, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "Indexer Service",
    about = "Standalone binary for the fuel indexer service.",
    version
)]
pub struct IndexerArgs {
    /// Log level passed to the Fuel Indexer service.
    #[clap(long, default_value = defaults::LOG_LEVEL, value_parser(["info", "debug", "error", "warn"]), help = "Log level passed to the Fuel Indexer service.")]
    pub log_level: String,

    /// Indexer service config file.
    #[clap(
        short,
        long,
        value_name = "FILE",
        help = "Indexer service config file."
    )]
    pub config: Option<PathBuf>,

    /// Indexer config file.
    #[clap(short, long, value_name = "FILE", help = "Indexer config file.")]
    pub manifest: Option<PathBuf>,

    /// Host of the running Fuel node.
    #[clap(
        long,
        help = "Host of the running Fuel node.",
        default_value = defaults::FUEL_NODE_HOST
    )]
    pub fuel_node_host: String,

    /// Listening port of the running Fuel node.
    #[clap(
        long,
        help = "Listening port of the running Fuel node.",
        default_value = defaults::FUEL_NODE_PORT
    )]
    pub fuel_node_port: String,

    /// Web API host.
    #[clap(long, help = "Web API host.", default_value = defaults::WEB_API_HOST)]
    pub web_api_host: String,

    /// Web API port.
    #[clap(long, help = "Web API port.", default_value = defaults::WEB_API_PORT)]
    pub web_api_port: String,

    /// Database type.
    #[clap(long, help = "Database type.", default_value = defaults::DATABASE, value_parser(["postgres"]))]
    pub database: String,

    /// Max body size for Web API requests.
    #[clap(long, help = "Max body size for Web API requests.", default_value_t = defaults::MAX_BODY_SIZE )]
    pub max_body_size: usize,

    /// Postgres username.
    #[clap(long, help = "Postgres username.")]
    pub postgres_user: Option<String>,

    /// Postgres database.
    #[clap(long, help = "Postgres database.")]
    pub postgres_database: Option<String>,

    /// Postgres password.
    #[clap(long, help = "Postgres password.")]
    pub postgres_password: Option<String>,

    /// Postgres host.
    #[clap(long, help = "Postgres host.")]
    pub postgres_host: Option<String>,

    /// Postgres port.
    #[clap(long, help = "Postgres port.")]
    pub postgres_port: Option<String>,

    /// Run database migrations before starting service.
    #[clap(long, help = "Run database migrations before starting service.")]
    pub run_migrations: bool,

    /// Use Prometheus metrics reporting.
    #[clap(long, help = "Use Prometheus metrics reporting.")]
    pub metrics: bool,

    /// Prevent indexers from running without handling any blocks.
    #[clap(
        long,
        help = "Prevent indexers from running without handling any blocks."
    )]
    pub stop_idle_indexers: bool,

    /// Automatically create and start database using provided options or defaults.
    #[clap(
        long,
        help = "Automatically create and start database using provided options or defaults."
    )]
    pub embedded_database: bool,

    /// Require users to authenticate for some operations.
    #[clap(long, help = "Require users to authenticate for some operations.")]
    pub auth_enabled: bool,

    /// Authentication scheme used.
    #[clap(long, help = "Authentication scheme used.")]
    pub auth_strategy: Option<String>,

    /// Secret used for JWT scheme (if JWT scheme is specified).
    #[clap(
        long,
        help = "Secret used for JWT scheme (if JWT scheme is specified)."
    )]
    pub jwt_secret: Option<String>,

    /// Issuer of JWT claims (if JWT scheme is specified).
    #[clap(long, help = "Issuer of JWT claims (if JWT scheme is specified).")]
    pub jwt_issuer: Option<String>,

    /// Amount of time (seconds) before expiring token (if JWT scheme is specified).
    #[clap(
        long,
        help = "Amount of time (seconds) before expiring token (if JWT scheme is specified)."
    )]
    pub jwt_expiry: Option<usize>,

    /// Enable verbose logging.
    #[clap(short, long, help = "Enable verbose logging.")]
    pub verbose: bool,

    /// Start a local Fuel node.
    #[clap(long, help = "Start a local Fuel node.")]
    pub local_fuel_node: bool,

    /// Allow network configuration via indexer manifests.
    #[clap(long, help = "Allow network configuration via indexer manifests.")]
    pub indexer_net_config: bool,

    /// Enable rate limiting.
    #[clap(long, help = "Enable rate limiting.")]
    pub rate_limit: bool,

    /// Maximum number of requests to allow over --rate-limit-window..
    #[clap(
        long,
        help = "Maximum number of requests to allow over --rate-limit-window.."
    )]
    pub rate_limit_request_count: Option<u64>,

    /// Number of seconds over which to allow --rate-limit-rps.
    #[clap(long, help = "Number of seconds over which to allow --rate-limit-rps.")]
    pub rate_limit_window_size: Option<u64>,

    /// The number of WASM opcodes after which the indexer's event handler will stop execution.
    #[clap(
        long,
        help = "The number of WASM opcodes after which the indexer's event handler will stop execution.",
        default_value_t = defaults::METERING_POINTS
    )]
    pub metering_points: u64,

    /// Whether to allow replacing an existing indexer. If not specified, an attempt to deploy over an existing indexer results in an error.
    #[clap(
        long,
        help = "Whether to allow replacing an existing indexer. If not specified, an attempt to deploy over an existing indexer results in an error."
    )]
    pub replace_indexer: bool,

    /// Allow the web API to accept raw SQL queries.
    #[clap(long, help = "Allow the web API to accept raw SQL queries.")]
    pub accept_sql_queries: bool,
}

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "Fuel Indexer API Server",
    about = "Fuel indexer Web API",
    version
)]
pub struct ApiServerArgs {
    /// Log level passed to the Fuel Indexer service.
    #[clap(long, default_value = defaults::LOG_LEVEL, value_parser(["info", "debug", "error", "warn"]), help = "Log level passed to the Fuel Indexer service.")]
    pub log_level: String,

    /// API server config file.
    #[clap(short, long, help = "API server config file.")]
    pub config: Option<PathBuf>,

    /// Host of the running Fuel node.
    #[clap(
        long,
        help = "Host of the running Fuel node.",
        default_value = defaults::FUEL_NODE_HOST
    )]
    pub fuel_node_host: String,

    /// Listening port of the running Fuel node.
    #[clap(
        long,
        help = "Listening port of the running Fuel node.",
        default_value = defaults::FUEL_NODE_PORT
    )]
    pub fuel_node_port: String,

    /// Web API host.
    #[clap(long, help = "Web API host.", default_value = defaults::WEB_API_HOST)]
    pub web_api_host: String,

    /// Web API port.
    #[clap(long, help = "Web API port.", default_value = defaults::WEB_API_PORT)]
    pub web_api_port: String,

    /// Database type.
    #[clap(long, help = "Database type.", default_value = defaults::DATABASE, value_parser(["postgres"]))]
    pub database: String,

    /// Max body size for Web API requests.
    #[clap(long, help = "Max body size for Web API requests.", default_value_t = defaults::MAX_BODY_SIZE )]
    pub max_body_size: usize,

    /// Run database migrations before starting service.
    #[clap(long, help = "Run database migrations before starting service.")]
    pub run_migrations: bool,

    /// Postgres username.
    #[clap(long, help = "Postgres username.")]
    pub postgres_user: Option<String>,

    /// Postgres database.
    #[clap(long, help = "Postgres database.")]
    pub postgres_database: Option<String>,

    /// Postgres password.
    #[clap(long, help = "Postgres password.")]
    pub postgres_password: Option<String>,

    /// Postgres host.
    #[clap(long, help = "Postgres host.")]
    pub postgres_host: Option<String>,

    /// Postgres port.
    #[clap(long, help = "Postgres port.")]
    pub postgres_port: Option<String>,

    /// Use Prometheus metrics reporting.
    #[clap(long, help = "Use Prometheus metrics reporting.")]
    pub metrics: bool,

    /// Require users to authenticate for some operations.
    #[clap(long, help = "Require users to authenticate for some operations.")]
    pub auth_enabled: bool,

    /// Authentication scheme used.
    #[clap(long, help = "Authentication scheme used.", value_parser(["jwt"]))]
    pub auth_strategy: Option<String>,

    /// Secret used for JWT scheme (if JWT scheme is specified).
    #[clap(
        long,
        help = "Secret used for JWT scheme (if JWT scheme is specified)."
    )]
    pub jwt_secret: Option<String>,

    /// Issuer of JWT claims (if JWT scheme is specified).
    #[clap(long, help = "Issuer of JWT claims (if JWT scheme is specified).")]
    pub jwt_issuer: Option<String>,

    /// Amount of time (seconds) before expiring token (if JWT scheme is specified).
    #[clap(
        long,
        help = "Amount of time (seconds) before expiring token (if JWT scheme is specified)."
    )]
    pub jwt_expiry: Option<usize>,

    /// Enable verbose logging.
    #[clap(short, long, help = "Enable verbose logging.")]
    pub verbose: bool,

    /// Enable rate limiting.
    #[clap(long, help = "Enable rate limiting.")]
    pub rate_limit: bool,

    /// Maximum number of requests to allow over --rate-limit-window..
    #[clap(
        long,
        help = "Maximum number of requests to allow over --rate-limit-window.."
    )]
    pub rate_limit_request_count: Option<u64>,

    /// Number of seconds over which to allow --rate-limit-rps.
    #[clap(long, help = "Number of seconds over which to allow --rate-limit-rps.")]
    pub rate_limit_window_size: Option<u64>,

    /// Allow the web API to accept raw SQL queries.
    #[clap(long, help = "Allow the web API to accept raw SQL queries.")]
    pub accept_sql_queries: bool,
}
