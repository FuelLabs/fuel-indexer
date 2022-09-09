use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
pub use clap::Parser;
use fuel_indexer_lib::{
    defaults,
    utils::{derive_socket_addr, is_env_var, trim_env_key},
};
use serde::Deserialize;
use std::fmt::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

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
            default_value = defaults::FUEL_NODE_HOST
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

pub trait MutableConfig {
    fn inject_env_vars(&mut self) -> Result<()>;
    fn derive_socket_addr(&self) -> Result<SocketAddr>;
    fn http_url(&self) -> String;
}

#[derive(Clone, Deserialize, Debug)]
pub struct FuelNodeConfig {
    pub host: String,
    pub port: String,
}

impl MutableConfig for FuelNodeConfig {
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

    fn derive_socket_addr(&self) -> Result<SocketAddr> {
        derive_socket_addr(&self.host, &self.port)
    }

    fn http_url(&self) -> String {
        http_url(&self.host, &self.port)
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

impl MutableConfig for DatabaseConfig {
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
                        *password = Some(
                            std::env::var(trim_env_key(pass))
                                .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &pass)),
                        );
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

                if is_env_var(database) {
                    *database = std::env::var(trim_env_key(database))
                        .unwrap_or_else(|_| format!("Failed to read '{}' from env", &database));
                }
            }
            DatabaseConfig::Sqlite { path } => {
                let os_str = path.as_os_str().to_str().unwrap();
                if is_env_var(os_str) {
                    *path = PathBuf::from(
                        std::env::var(trim_env_key(os_str))
                            .unwrap_or_else(|_| format!("Failed to read '{}' from env", os_str)),
                    );
                }
            }
        }
        Ok(())
    }

    fn derive_socket_addr(&self) -> Result<SocketAddr> {
        match self {
            DatabaseConfig::Postgres { host, port, .. } => derive_socket_addr(host, port),
            _ => {
                panic!("Cannot use MutableConfig::derive_socket_addr on a SQLite database.")
            }
        }
    }

    fn http_url(&self) -> String {
        todo!()
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

impl MutableConfig for GraphQLConfig {
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

    fn derive_socket_addr(&self) -> Result<SocketAddr> {
        derive_socket_addr(&self.host, &self.port)
    }

    fn http_url(&self) -> String {
        http_url(&self.host, &self.port)
    }
}

#[derive(Clone, Deserialize, Default, Debug)]
pub struct IndexerConfig {
    pub fuel_node: FuelNodeConfig,
    pub graphql_api: GraphQLConfig,
    pub database: DatabaseConfig,
}

#[derive(Deserialize)]
pub struct TmpIndexerConfig {
    pub fuel_node: Option<FuelNodeConfig>,
    pub graphql_api: Option<GraphQLConfig>,
    pub database: Option<DatabaseConfig>,
}

impl IndexerConfig {
    // FIXME
    pub fn upgrade_optionals(&mut self, tmp: TmpIndexerConfig) {
        if let Some(cfg) = tmp.fuel_node {
            self.fuel_node = cfg;
        }

        if let Some(cfg) = tmp.database {
            self.database = cfg;
        }

        if let Some(cfg) = tmp.graphql_api {
            self.graphql_api = cfg;
        }
    }

    pub fn from_opts(args: IndexerArgs) -> IndexerConfig {
        let database = match args.database.as_str() {
            "postgres" => DatabaseConfig::Postgres {
                user: args.postgres_user,
                password: args.postgres_password,
                host: args.postgres_host,
                port: args.postgres_port,
                database: args.postgres_database,
            },
            "sqlite" => DatabaseConfig::Sqlite {
                path: args.sqlite_database.into(),
            },
            _ => {
                panic!("Unrecognized database type in options.");
            }
        };

        let config = IndexerConfig {
            database,
            fuel_node: FuelNodeConfig {
                host: args.fuel_node_host,
                port: args.fuel_node_port,
            },
            graphql_api: GraphQLConfig {
                host: args.graphql_api_host,
                port: args.graphql_api_port,
                run_migrations: args.run_migrations,
            },
        };

        config
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let mut config = IndexerConfig::default();

        config.inject_env_vars();

        Ok(config)
    }

    pub fn inject_env_vars(&mut self) {
        let _ = self.fuel_node.inject_env_vars();
        let _ = self.database.inject_env_vars();
        let _ = self.graphql_api.inject_env_vars();
    }
}
