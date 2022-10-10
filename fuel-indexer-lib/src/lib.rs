pub mod config;
pub mod manifest;
pub mod utils;

pub mod defaults {

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

    pub const SQLITE_DATABASE: &str = "sqlite.db";

    pub const GRAPHQL_API_RUN_MIGRATIONS: Option<bool> = None;

    pub const ASSET_REFRESH_CHANNEL_SIZE: usize = 100;
}
