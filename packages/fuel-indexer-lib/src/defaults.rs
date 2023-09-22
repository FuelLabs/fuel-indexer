/// Host of the running Fuel node.
pub const FUEL_NODE_HOST: &str = "localhost";

/// Listening port of the running Fuel node.
pub const FUEL_NODE_PORT: &str = "4000";

/// GraphQL API host.
pub const WEB_API_HOST: &str = "localhost";

/// GraphQL API port.
pub const WEB_API_PORT: &str = "29987";

/// Database type.
pub const DATABASE: &str = "postgres";

/// Postgres database.
pub const POSTGRES_DATABASE: &str = "postgres";

/// Postgres username.
pub const POSTGRES_USER: &str = "postgres";

/// Postgres host.
pub const POSTGRES_HOST: &str = "localhost";

/// Postgres port.
pub const POSTGRES_PORT: &str = "5432";

/// Postgres password.
pub const POSTGRES_PASSWORD: &str = "postgres";

/// Number of failed calls to allow before stopping the indexer.
pub const INDEXER_FAILED_CALLS: usize = 10;

/// Stop indexers that have seen `MAX_CONSECUTIVE_EMPTY_BLOCK_RESPONSES`.
pub const STOP_IDLE_INDEXERS: bool = false;

/// Max body size for GraphQL API requests (5 MB).
pub const MAX_BODY_SIZE: usize = 5242880;

/// Size of web-API-to-executor-service channel.
pub const SERVICE_REQUEST_CHANNEL_SIZE: usize = 100;

/// How long to wait if request to the Fuel GQL client returned no data.
pub const IDLE_SERVICE_WAIT_SECS: u64 = 1;

/// How many times to retry connecting to the database.
pub const MAX_DATABASE_CONNECTION_ATTEMPTS: usize = 5;

/// How long to initially wait before retrying to connect to the database.
pub const INITIAL_RETRY_DELAY_SECS: u64 = 2;

/// Use an embedded PostgresQL database.
pub const EMBEDDED_DATABASE: bool = false;

/// If using `--stop-idle-indexers`, this is the number of empty blocks after which the executor should stop.
pub const MAX_CONSECUTIVE_EMPTY_BLOCK_RESPONSES: usize = 10;

/// Amount of time to wait before fetching new blocks, if an executor error occurs.
pub const DELAY_FOR_SERVICE_ERROR: u64 = 1;

/// Amount of time to wait before fetching new blocks, if the Fuel GQL client returns no data.
pub const DELAY_FOR_EMPTY_PAGE: u64 = 1;

/// Run migrations on startup.
pub const RUN_MIGRATIONS: bool = true;

/// Make a metrics endpoint available at `/api/metrics`.
pub const USE_METRICS: bool = true;

/// Directory at which general Fuel assets and metadata are stored.
pub const FUEL_HOME_DIR: &str = ".fuel";

/// Directory inside `FUEL_HOME_DIR` at which indexer assets and metadata are stored.
pub const INDEXER_CONFIG_DIR: &str = "indexer";

/// Binary name for `forc index` CLI.
pub const FORC_INDEX: &str = "forc-index";

/// Require users to authenticate for some operations.
pub const AUTH_ENABLED: bool = false;

/// Amount of time (seconds) before expiring token (if JWT scheme is specified).
pub const JWT_EXPIRY_SECS: usize = 2592000;

/// JWT secret (should not be used in production)
pub const JWT_SECRET: &str = "abcdefghijklmnopqrstuvwxyz1234567890";

/// Authentication strategy
pub const AUTH_STRATEGY: &str = "jwt";

/// JWT issuer
pub const JWT_ISSUER: &str = "FuelLabs";

/// Index of account to use for signing.
pub const ACCOUNT_INDEX: &str = "0";

/// Verbose output.
pub const VERBOSE_LOGGING: bool = false;

/// Verbose output for database operations.
pub const VERBOSE_DB_LOGGING: &str = "false";

/// Amount of blocks to return in a request to a Fuel node.
pub const NODE_BLOCK_PAGE_SIZE: usize = 20;

/// Start a local Fuel node.
pub const LOCAL_FUEL_NODE: bool = false;

/// Allow indexers to connect to the Fuel node specified in their respective manifests.
pub const INDEXER_NET_CONFIG: bool = false;

/// Enable rate limiting.
pub const RATE_LIMIT_ENABLED: bool = false;

/// Maximum number of requests to allow over --rate-limit-window..
pub const RATE_LIMIT_REQUEST_COUNT: u64 = 10;

/// Number of seconds over which to allow --rate-limit-rps.
pub const RATE_LIMIT_WINDOW_SIZE: u64 = 5;

/// Log level.
pub const LOG_LEVEL: &str = "info";

/// The number of WASM opcodes after which the indexer's `handle_events` function will stop execution.
pub const METERING_POINTS: u64 = 30_000_000_000;

/// Whether to allow replacing an indexer.
///
/// If this is disabled, then an HTTP 409 Conflict will be returned if an indexer with the same name already exists.
pub const REPLACE_INDEXER: bool = false;

/// Whether to remove the indexed data when replacing an indexer.
pub const REMOVE_DATA: bool = false;

/// Allow the web server to accept raw SQL queries.
pub const ACCEPT_SQL: bool = false;

/// Store blocks in the database and use these stored blocks to fast-forward an indexer starting up.
pub const ENABLE_BLOCK_STORE: bool = false;
