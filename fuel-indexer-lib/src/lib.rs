pub mod utils {

    use anyhow::Result;
    use std::net::{SocketAddr, ToSocketAddrs};
    use tracing::{debug, warn};

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
                    .unwrap_or_else(|_| panic!("Unable to resolve domain for '{}'", host))
                    .collect();

                let addr = addrs
                    .pop()
                    .unwrap_or_else(|| panic!("Could not derive SocketAddr from '{}'", host));

                debug!("Parsed SocketAddr '{}' from '{}'", addr.to_string(), host);

                Ok(addr)
            }
        }
    }
}

pub mod defaults {
    pub const FUEL_NODE_HOST: &str = "127.0.0.1";
    pub const FUEL_NODE_PORT: &str = "4000";

    pub const GRAPHQL_API_HOST: &str = "0.0.0.0";
    pub const GRAPHQL_API_PORT: &str = "29987";

    pub const POSTGRES_USER: &str = "postgres";
    pub const POSTGRES_HOST: &str = "127.0.0.1";
    pub const POSTGRES_PORT: &str = "5432";
}

pub mod config {
    use crate::{
        defaults,
        utils::{is_env_var, trim_env_key},
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
        #[clap(short, long, help = "Run local test node")]
        pub local: bool,
        #[clap(short, long, parse(from_os_str), help = "Indexer service config file")]
        pub config: Option<PathBuf>,
        #[clap(short, long, parse(from_os_str), help = "Indexer service config file")]
        pub test_manifest: Option<PathBuf>,
        #[clap(
            long,
            help = "Listening IP of the running Fuel node. (default = '127.0.0.1')"
        )]
        pub fuel_node_host: Option<String>,
        #[clap(long, help = "Listening port of the running Fuel node.")]
        pub fuel_node_port: Option<String>,
        #[clap(long, help = "GraphQL API IP. (default = '0.0.0.0')")]
        pub graphql_api_host: Option<String>,
        #[clap(long, help = "GraphQL API port. (default = 29987)")]
        pub graphql_api_port: Option<String>,
        #[clap(long, help = "Database type", default_value = "postgres", value_parser(["postgres", "sqlite"]))]
        pub database: String,
        #[clap(long, default_value = "sqlite.db", help = "Sqlite database.")]
        pub sqlite_database: String,
        #[clap(long, help = "Postgres username. (default = 'postgres')")]
        pub postgres_user: Option<String>,
        #[clap(long, help = "Postgres database. (default = 'postgres')")]
        pub postgres_database: Option<String>,
        #[clap(long, help = "Postgres password.")]
        pub postgres_password: Option<String>,
        #[clap(long, help = "Postgres host. (default = '127.0.0.1')")]
        pub postgres_host: Option<String>,
        #[clap(long, help = "Postgres port. (default = 5432)")]
        pub postgres_port: Option<String>,
    }

    #[derive(Debug, Parser, Clone)]
    #[clap(name = "Indexer API Service", about = "Fuel Indexer GraphQL API")]
    pub struct ApiServerArgs {
        #[clap(short, long, help = "API Server config.")]
        pub config: Option<PathBuf>,
        #[clap(long, help = "GraphQL API IP. (default = '0.0.0.0')")]
        pub graphql_api_host: Option<String>,
        #[clap(long, help = "GraphQL API port. (default = 29987)")]
        pub graphql_api_port: Option<String>,
        #[clap(long, help = "Database type", default_value = "postgres", value_parser(["postgres", "sqlite"]))]
        pub database: String,
        #[clap(long, default_value = "sqlite.db", help = "Sqlite database.")]
        pub sqlite_database: PathBuf,
        #[clap(long, help = "Postgres username. (default = 'postgres')")]
        pub postgres_user: Option<String>,
        #[clap(long, help = "Postgres database. (default = 'postgres')")]
        pub postgres_database: Option<String>,
        #[clap(long, help = "Postgres password.")]
        pub postgres_password: Option<String>,
        #[clap(long, help = "Postgres host. (default = '127.0.0.1')")]
        pub postgres_host: Option<String>,
        #[clap(long, help = "Postgres port. (default = 5432)")]
        pub postgres_port: Option<String>,
    }

    pub trait InjectEnvironment {
        fn inject_env_vars(&mut self) -> Result<()>;
    }

    #[derive(Clone, Deserialize, Debug)]
    pub struct FuelNodeConfig {
        pub host: String,
        pub port: String,
    }

    impl InjectEnvironment for FuelNodeConfig {
        fn inject_env_vars(&mut self) -> Result<()> {
            if is_env_var(&self.host) {
                self.host = std::env::var(trim_env_key(&self.host))
                    .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.host));
            }

            if is_env_var(&self.port) {
                self.port = std::env::var(trim_env_key(&self.port))
                    .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.port));
            }

            Ok(())
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
    pub enum DatabaseConfig {
        Sqlite {
            path: PathBuf,
        },
        Postgres {
            user: String,
            password: Option<String>,
            host: String,
            port: String,
            database: Option<String>,
        },
    }

    impl InjectEnvironment for DatabaseConfig {
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
                            .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &user));
                    }

                    if let Some(pass) = &password {
                        if is_env_var(pass) {
                            *password =
                                Some(std::env::var(trim_env_key(pass)).unwrap_or_else(|_| {
                                    panic!("Failed to read '{}' from env", &pass)
                                }));
                        }
                    }

                    if is_env_var(host) {
                        *host = std::env::var(trim_env_key(host))
                            .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &host));
                    }

                    if is_env_var(port) {
                        *port = std::env::var(trim_env_key(port))
                            .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &port));
                    }

                    if let Some(db) = &database {
                        if is_env_var(db) {
                            *database =
                                Some(std::env::var(trim_env_key(db)).unwrap_or_else(|_| {
                                    format!("Failed to read '{}' from env", &db)
                                }));
                        }
                    }
                }
                DatabaseConfig::Sqlite { path } => {
                    let os_str = path.as_os_str().to_str().unwrap();
                    if is_env_var(os_str) {
                        *path =
                            PathBuf::from(std::env::var(trim_env_key(os_str)).unwrap_or_else(
                                |_| format!("Failed to read '{}' from env", os_str),
                            ));
                    }
                }
            }
            Ok(())
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

                    let _ = write!(uri, "@{}:{}", host, port);

                    if let Some(db_name) = &database {
                        let _ = write!(uri, "/{}", db_name);
                    }

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
                database: None,
            }
        }
    }

    #[derive(Clone, Deserialize, Debug)]
    pub struct GraphQLConfig {
        pub host: String,
        pub port: String,
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

    impl InjectEnvironment for GraphQLConfig {
        fn inject_env_vars(&mut self) -> Result<()> {
            if is_env_var(&self.host) {
                self.host = std::env::var(trim_env_key(&self.host))
                    .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.host));
            }

            if is_env_var(&self.port) {
                self.port = std::env::var(trim_env_key(&self.port))
                    .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.port));
            }

            Ok(())
        }
    }
}
