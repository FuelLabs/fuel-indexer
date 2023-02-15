pub const FUEL_NODE_HOST: &str = "127.0.0.1";
pub const FUEL_NODE_PORT: &str = "4000";

pub const GRAPHQL_API_HOST: &str = "127.0.0.1";
pub const GRAPHQL_API_PORT: &str = "29987";

pub const DATABASE: &str = "postgres";
pub const POSTGRES_DATABASE: &str = "postgres";
pub const POSTGRES_USER: &str = "postgres";
pub const POSTGRES_HOST: &str = "127.0.0.1";
pub const POSTGRES_PORT: &str = "5432";
pub const POSTGRES_PASSWORD: &str = "";

pub const INDEX_FAILED_CALLS: usize = 10;

pub const GRAPHQL_API_RUN_MIGRATIONS: bool = false;
pub const MAX_BODY_LIMIT: &str = "10"; // 5MB in bytes

pub const SERVICE_REQUEST_CHANNEL_SIZE: usize = 100;
pub const IDLE_SERVICE_WAIT_SECS: u64 = 3;

pub const MAX_DATABASE_CONNECTION_ATTEMPTS: usize = 5;
pub const INITIAL_RETRY_DELAY_SECS: u64 = 2;
pub const MAX_EMPTY_BLOCK_REQUESTS: usize = 10;

pub const DELAY_FOR_SERVICE_ERR: u64 = 5;
pub const DELAY_FOR_EMPTY_PAGE: u64 = 1;
