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
    pub struct PostgresConfig {
        pub user: String,
        pub password: Option<String>,
        pub host: String,
        pub port: String,
        pub database: Option<String>,
    }

    impl InjectEnvironment for PostgresConfig {
        fn inject_env_vars(&mut self) -> Result<()> {
            if is_env_var(&self.user) {
                self.user = std::env::var(trim_env_key(&self.user))
                    .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.user));
            }

            if let Some(password) = &self.password {
                if is_env_var(password) {
                    self.password = Some(
                        std::env::var(trim_env_key(password))
                            .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &password)),
                    );
                }
            }

            if is_env_var(&self.host) {
                self.host = std::env::var(trim_env_key(&self.host))
                    .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.host));
            }

            if is_env_var(&self.port) {
                self.port = std::env::var(trim_env_key(&self.port))
                    .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.port));
            }

            if let Some(database) = &self.database {
                if is_env_var(database) {
                    self.database =
                        Some(std::env::var(trim_env_key(database)).unwrap_or_else(|_| {
                            format!("Failed to read '{}' from env", &database)
                        }));
                }
            }

            Ok(())
        }
    }

    impl std::string::ToString for PostgresConfig {
        fn to_string(&self) -> String {
            let mut uri: String = format!("postgres://{}", self.user);

            if let Some(pass) = &self.password {
                let _ = write!(uri, ":{}", pass);
            }

            let _ = write!(uri, "@{}:{}", self.host, self.port);

            if let Some(db_name) = &self.database {
                let _ = write!(uri, "/{}", db_name);
            }

            uri
        }
    }

    impl std::fmt::Debug for PostgresConfig {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let _ = f
                .debug_struct("PostgresConfig")
                .field("user", &self.user)
                .field("password", &"XXXX")
                .field("host", &self.host)
                .field("port", &self.port)
                .field("database", &self.database)
                .finish();

            Ok(())
        }
    }

    impl Default for PostgresConfig {
        fn default() -> Self {
            Self {
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
