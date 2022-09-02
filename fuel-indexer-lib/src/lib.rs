pub mod utils {

    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use std::net::{SocketAddr, ToSocketAddrs};
    use tracing::{info, warn};

    pub fn trim_env_key(key: &str) -> &str {
        // Abmiguous key: $FOO, non-ambiguous key: ${FOO}
        let not_ambiguous = key.starts_with("${");
        match not_ambiguous {
            false => &key[1..],
            true => &key[2..key.len() - 1],
        }
    }

    pub fn is_env_var(key: &str) -> bool {
        key.starts_with('$') || (key.starts_with("${") && key.ends_with('}'))
    }

    pub fn derive_socket_addr(host: &String, port: &String) -> Result<SocketAddr> {
        let host = format!("{}:{}", host, port);
        match &host.parse() {
            Ok(sock) => Ok(*sock),
            Err(e) => {
                warn!(
                    "Failed to parse '{}' as a SocketAddr due to '{}'. Retrying using ToSocketAddrs.",
                    host, e
                );

                let mut addrs: Vec<_> = host
                    .to_socket_addrs()
                    .expect_or_else(|_| panic!("Unable to resolve domain for '{}'", host))
                    .collect();

                let addr = addrs
                    .pop()
                    .expect_or_else(|| panic!("Could not derive SocketAddr from '{}'", host));

                info!("Parsed SocketAddr '{}' from '{}'", addr.to_string(), host);

                Ok(addr)
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ServiceStatus {
        OK,
        NotOk,
    }

    impl From<FuelNodeHealthResponse> for ServiceStatus {
        fn from(r: FuelNodeHealthResponse) -> Self {
            match r.up {
                true => ServiceStatus::OK,
                _ => ServiceStatus::NotOk,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct FuelNodeHealthResponse {
        up: bool,
    }
}

pub mod defaults {
    pub const FUEL_NODE_HOST: &str = "127.0.0.1";
    pub const FUEL_NODE_PORT: &str = "4000";

    pub const GRAPHQL_API_HOST: &str = "0.0.0.0";
    pub const GRAPHQL_API_PORT: &str = "29987";

    pub const DATABASE: &str = "postgres";
    pub const POSTGRES_DATABASE: &str = "postgres";
    pub const POSTGRES_USER: &str = "postgres";
    pub const POSTGRES_HOST: &str = "127.0.0.1";
    pub const POSTGRES_PORT: &str = "5432";

    pub const SQLITE_DATABASE: &str = "sqlite.db";

    pub const GRAPHQL_API_RUN_MIGRATIONS: Option<bool> = None;

    pub const PROTOCOL: &str = "http";
}

pub mod config {
    use crate::{
        defaults,
        utils::{derive_socket_addr, is_env_var, trim_env_key},
    };
    use anyhow::Result;
    pub use clap::Parser;
    use serde::Deserialize;
    use std::fmt::Write;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    #[derive(Debug, Parser, Clone)]
    #[clap(
        name = "Indexer Service",
        about = "Standalone binary for the fuel indexer service"
    )]
    pub struct IndexerArgs {
        #[clap(short, long, help = "Run local test node.")]
        pub local: bool,
        #[clap(short, long, parse(from_os_str), help = "Indexer service config file.")]
        pub config: Option<PathBuf>,
        #[clap(
            short,
            long,
            parse(from_os_str),
            help = "Manifest file used to bootstrap the indexer service."
        )]
        pub manifest: Option<PathBuf>,
        #[clap(
            long,
            help = "Listening IP of the running Fuel node.)",
            default_value = defaults::FUEL_NODE_HOST,
        )]
        pub fuel_node_host: String,
        #[clap(long, help = "Listening port of the running Fuel node.", default_value = defaults::FUEL_NODE_PORT)]
        pub fuel_node_port: String,
        #[clap(long, help = "GraphQL API IP.", default_value = defaults::GRAPHQL_API_HOST )]
        pub graphql_api_host: String,
        #[clap(long, help = "GraphQL API port.", default_value = defaults::GRAPHQL_API_PORT )]
        pub graphql_api_port: String,
        #[clap(long, help = "Database type.", default_value = defaults::DATABASE, value_parser(["postgres", "sqlite"]))]
        pub database: String,
        #[clap(long, help = "Sqlite database.", default_value = defaults::SQLITE_DATABASE)]
        pub sqlite_database: PathBuf,
        #[clap(long, help = "Postgres username.", default_value = defaults::POSTGRES_USER)]
        pub postgres_user: String,
        #[clap(long, help = "Postgres database.", default_value = defaults::POSTGRES_DATABASE )]
        pub postgres_database: String,
        #[clap(long, help = "Postgres password.")]
        pub postgres_password: Option<String>,
        #[clap(long, help = "Postgres host.", default_value = defaults::POSTGRES_HOST)]
        pub postgres_host: String,
        #[clap(long, help = "Postgres port.", default_value = defaults::POSTGRES_PORT)]
        pub postgres_port: String,
        #[clap(long, help = "Run database migrations for the GraphQL API service.")]
        pub run_migrations: Option<bool>,
    }

    #[derive(Debug, Parser, Clone)]
    #[clap(name = "Indexer API Service", about = "Fuel Indexer GraphQL API")]
    pub struct ApiServerArgs {
        #[clap(short, long, help = "API Server config.")]
        pub config: Option<PathBuf>,
        #[clap(long, help = "GraphQL API IP.", default_value = defaults::GRAPHQL_API_HOST)]
        pub graphql_api_host: String,
        #[clap(long, help = "GraphQL API port.", default_value = defaults::GRAPHQL_API_PORT)]
        pub graphql_api_port: String,
        #[clap(long, help = "Database type", default_value = defaults::DATABASE, value_parser(["postgres", "sqlite"]))]
        pub database: String,
        #[clap(long, default_value = defaults::SQLITE_DATABASE, help = "Sqlite database.")]
        pub sqlite_database: PathBuf,
        #[clap(long, help = "Postgres username.", default_value = defaults::POSTGRES_USER)]
        pub postgres_user: String,
        #[clap(long, help = "Postgres database.", default_value = defaults::POSTGRES_DATABASE)]
        pub postgres_database: String,
        #[clap(long, help = "Postgres password.")]
        pub postgres_password: Option<String>,
        #[clap(long, help = "Postgres host.", default_value = defaults::POSTGRES_HOST)]
        pub postgres_host: String,
        #[clap(long, help = "Postgres port.", default_value = defaults::POSTGRES_PORT)]
        pub postgres_port: String,
    }

    fn http_url(host: &String, port: &String) -> String {
        let protocol = match port.as_str() {
            "443" | "4443" => "https",
            _ => "http",
        };

        format!("{}://{}:{}", protocol, host, port)
    }

    pub trait AdjustableConfig {
        fn inject_env_vars(&mut self) -> Result<()>;
        fn derive_socket_addr(&self) -> Result<SocketAddr>;
    }

    #[derive(Clone, Deserialize, Debug)]
    pub struct FuelNodeConfig {
        pub host: String,
        pub port: String,
    }

    impl FuelNodeConfig {
        pub fn http_url(&self) -> String {
            http_url(&self.host, &self.port)
        }
    }

    impl AdjustableConfig for FuelNodeConfig {
        fn inject_env_vars(&mut self) -> Result<()> {
            if is_env_var(&self.host) {
                self.host = std::env::var(trim_env_key(&self.host))
                expect_or_else(|_| panic!("Failed to read '{}' from env", &self.host));
            }

            if is_env_var(&self.port) {
                self.port = std::env::var(trim_env_key(&self.port))
                    .expect_or_else(|_| panic!("Failed to read '{}' from env", &self.port));
            }

            Ok(())
        }

        fn derive_socket_addr(&self) -> Result<SocketAddr> {
            derive_socket_addr(&self.host, &self.port)
        }
    }

    impl Default for FuelNodeConfig {
        fn default() -> Self {
            Self {
                host: defaults::FUEL_NODE_HOST.into(),
                port: defaults::FUEL_NODE_PORT.into(),
            }
        }
    }

    impl From<SocketAddr> for FuelNodeConfig {
        fn from(s: SocketAddr) -> FuelNodeConfig {
            let parts: Vec<String> = s.to_string().split(':').map(|x| x.to_owned()).collect();
            let host = parts[0].to_owned();
            let port = parts[1].to_owned();
            FuelNodeConfig { host, port }
        }
    }

    impl std::string::ToString for FuelNodeConfig {
        fn to_string(&self) -> String {
            format!("{}:{}", self.host, self.port)
        }
    }

    #[derive(Clone, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum DatabaseConfig {
        Sqlite {
            path: PathBuf,
        },
        Postgres {
            user: String,
            password: Option<String>,
            host: String,
            port: String,
            database: String,
        },
    }

    impl AdjustableConfig for DatabaseConfig {
        fn inject_env_vars(&mut self) -> Result<()> {
            match self {
                DatabaseConfig::Postgres {
                    user,
                    password,
                    host,
                    port,
                    database,
                } => {
                    if is_env_var(user) {
                        *user = std::env::var(trim_env_key(user))
                        expect_or_else(|_| panic!("Failed to read '{}' from env", &user));
                    }

                    if let Some(pass) = &password {
                        if is_env_var(pass) {
                            *password =
                                Some(std::env::var(trim_env_key(pass)).expect_or_else(|_| {
                                    panic!("Failed to read '{}' from env", &pass)
                                }));
                        }
                    }

                    if is_env_var(host) {
                        *host = std::env::var(trim_env_key(host))
                            .expect_or_else(|_| panic!("Failed to read '{}' from env", &host));
                    }

                    if is_env_var(port) {
                        *port = std::env::var(trim_env_key(port))
                            .expect_or_else(|_| panic!("Failed to read '{}' from env", &port));
                    }
                    if is_env_var(database) {
                        *database = std::env::var(trim_env_key(database))
                            .expect_or_else(|_| format!("Failed to read '{}' from env", &database));
                    }
                }
                DatabaseConfig::Sqlite { path } => {
                    let os_str = path.as_os_str().to_str().expect();
                    if is_env_var(os_str) {
                        *path =
                            PathBuf::from(std::env::var(trim_env_key(os_str)).expect_or_else(
                                |_| format!("Failed to read '{}' from env", os_str),
                            ));
                    }
                }
            }
            Ok(())
        }

        fn derive_socket_addr(&self) -> Result<SocketAddr> {
            match self {
                DatabaseConfig::Postgres { host, port, .. } => derive_socket_addr(host, port),
                _ => {
                    panic!("Cannot use AdjustableConfig::derive_socket_addr on a SQLite database.")
                }
            }
        }
    }

    impl std::string::ToString for DatabaseConfig {
        fn to_string(&self) -> String {
            match self {
                DatabaseConfig::Postgres {
                    user,
                    password,
                    host,
                    port,
                    database,
                } => {
                    let mut uri: String = format!("postgres://{}", user);

                    if let Some(pass) = &password {
                        let _ = write!(uri, ":{}", pass);
                    }

                    let _ = write!(uri, "@{}:{}/{}", host, port, database);

                    uri
                }
                DatabaseConfig::Sqlite { path } => {
                    format!("sqlite://{}", path.display())
                }
            }
        }
    }

    impl std::fmt::Debug for DatabaseConfig {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DatabaseConfig::Postgres {
                    user,
                    host,
                    port,
                    database,
                    ..
                } => {
                    let _ = f
                        .debug_struct("PostgresConfig")
                        .field("user", &user)
                        .field("password", &"XXXX")
                        .field("host", &host)
                        .field("port", &port)
                        .field("database", &database)
                        .finish();
                }
                DatabaseConfig::Sqlite { path } => {
                    let _ = f.debug_struct("SqliteConfig").field("path", &path).finish();
                }
            }

            Ok(())
        }
    }

    impl Default for DatabaseConfig {
        fn default() -> Self {
            DatabaseConfig::Postgres {
                user: defaults::POSTGRES_USER.into(),
                password: None,
                host: defaults::POSTGRES_HOST.into(),
                port: defaults::POSTGRES_PORT.into(),
                database: defaults::POSTGRES_DATABASE.into(),
            }
        }
    }

    #[derive(Clone, Deserialize, Debug)]
    pub struct GraphQLConfig {
        pub host: String,
        pub port: String,
        pub run_migrations: Option<bool>,
    }

    impl GraphQLConfig {
        pub fn http_url(&self) -> String {
            http_url(&self.host, &self.port)
        }
    }

    impl std::string::ToString for GraphQLConfig {
        fn to_string(&self) -> String {
            format!("{}:{}", self.host, self.port)
        }
    }

    impl Default for GraphQLConfig {
        fn default() -> Self {
            Self {
                host: defaults::GRAPHQL_API_HOST.into(),
                port: defaults::GRAPHQL_API_PORT.into(),
                run_migrations: defaults::GRAPHQL_API_RUN_MIGRATIONS,
            }
        }
    }

    impl From<GraphQLConfig> for SocketAddr {
        fn from(cfg: GraphQLConfig) -> SocketAddr {
            format!("{}:{}", cfg.host, cfg.port)
                .parse()
                .expect("Failed")
        }
    }

    impl AdjustableConfig for GraphQLConfig {
        fn inject_env_vars(&mut self) -> Result<()> {
            if is_env_var(&self.host) {
                self.host = std::env::var(trim_env_key(&self.host))
                    .expect_or_else(|_| panic!("Failed to read '{}' from env", &self.host));
            }

            if is_env_var(&self.port) {
                self.port = std::env::var(trim_env_key(&self.port))
                    .expect_or_else(|_| panic!("Failed to read '{}' from env", &self.port));
            }

            Ok(())
        }

        fn derive_socket_addr(&self) -> Result<SocketAddr> {
            derive_socket_addr(&self.host, &self.port)
        }
    }
}
