use crate::{
    defaults,
    utils::{derive_socket_addr, is_opt_env_var, trim_opt_env_key},
};
use anyhow::Result;
pub use clap::Parser;
use serde::Deserialize;
use std::fs::File;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use strum::{AsRefStr, EnumString};

#[derive(Debug, EnumString, AsRefStr)]
pub enum EnvVar {
    #[strum(serialize = "POSTGRES_HOST")]
    PostgresHost,
    #[strum(serialize = "POSTGRES_PASSWORD")]
    PostgresPassword,
    #[strum(serialize = "POSTGRES_DATABASE")]
    PostgresDatabase,
    #[strum(serialize = "POSTGRES_PORT")]
    PostgresPort,
    #[strum(serialize = "POSTGRES_USER")]
    PostgresUser,
}

pub fn env_or_default(var: EnvVar, default: String) -> String {
    std::env::var(var.as_ref()).unwrap_or(default)
}

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "Indexer Service",
    about = "Standalone binary for the fuel indexer service"
)]
pub struct IndexerArgs {
    #[clap(short, long, parse(from_os_str), help = "Indexer service config file")]
    pub config: Option<PathBuf>,
    #[clap(short, long, parse(from_os_str), help = "Indexer service config file")]
    pub manifest: Option<PathBuf>,
    #[clap(
        long,
        help = "Listening IP of the running Fuel node.",
        default_value = defaults::FUEL_NODE_HOST
    )]
    pub fuel_node_host: String,
    #[clap(
        long,
        help = "Listening port of the running Fuel node.",
        default_value = defaults::FUEL_NODE_PORT
    )]
    pub fuel_node_port: String,
    #[clap(long, help = "GraphQL API IP.", default_value = defaults::GRAPHQL_API_HOST)]
    pub graphql_api_host: String,
    #[clap(long, help = "GraphQL API port.", default_value = defaults::GRAPHQL_API_PORT)]
    pub graphql_api_port: String,
    #[clap(long, help = "Database type.", default_value = defaults::DATABASE, value_parser(["postgres", "sqlite"]))]
    pub database: String,
    #[clap(long, help = "Sqlite database.", default_value = defaults::SQLITE_DATABASE)]
    pub sqlite_database: PathBuf,
    #[clap(long, help = "Postgres username.")]
    pub postgres_user: Option<String>,
    #[clap(long, help = "Postgres database.")]
    pub postgres_database: Option<String>,
    #[clap(long, help = "Postgres password.")]
    pub postgres_password: Option<String>,
    #[clap(long, help = "Postgres host.")]
    pub postgres_host: Option<String>,
    #[clap(long, help = "Postgres port.")]
    pub postgres_port: Option<String>,
    #[clap(long, help = "Run database migrations for the GraphQL API service.")]
    pub run_migrations: Option<bool>,
    #[clap(long, help = "Use Prometheus metrics reporting.")]
    pub metrics: Option<bool>,
}

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer API Service", about = "Fuel indexer GraphQL API")]
pub struct ApiServerArgs {
    #[clap(short, long, help = "API Server config.")]
    pub config: Option<PathBuf>,
    #[clap(long, help = "GraphQL API IP.", default_value = defaults::GRAPHQL_API_HOST)]
    pub graphql_api_host: String,
    #[clap(long, help = "GraphQL API port.", default_value = defaults::GRAPHQL_API_PORT)]
    pub graphql_api_port: String,
    #[clap(long, help = "Postgres username.")]
    pub postgres_user: Option<String>,
    #[clap(long, help = "Postgres database.")]
    pub postgres_database: Option<String>,
    #[clap(long, help = "Postgres password.")]
    pub postgres_password: Option<String>,
    #[clap(long, help = "Postgres host.")]
    pub postgres_host: Option<String>,
    #[clap(long, help = "Postgres port.")]
    pub postgres_port: Option<String>,
}

fn derive_http_url(host: &String, port: &String) -> String {
    let protocol = match port.as_str() {
        "443" | "4443" => "https",
        _ => "http",
    };

    format!("{}://{}:{}", protocol, host, port)
}

pub trait MutableConfig {
    fn inject_opt_env_vars(&mut self) -> Result<()>;
    fn derive_socket_addr(&self) -> SocketAddr;
    fn derive_http_url(&self) -> String;
}

#[derive(Clone, Deserialize, Debug)]
pub struct FuelNodeConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: String,
}

impl MutableConfig for FuelNodeConfig {
    fn inject_opt_env_vars(&mut self) -> Result<()> {
        if is_opt_env_var(&self.host) {
            self.host = std::env::var(trim_opt_env_key(&self.host))
                .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.host));
        }

        if is_opt_env_var(&self.port) {
            self.port = std::env::var(trim_opt_env_key(&self.port))
                .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.port));
        }

        Ok(())
    }

    fn derive_socket_addr(&self) -> SocketAddr {
        derive_socket_addr(&self.host, &self.port)
    }

    fn derive_http_url(&self) -> String {
        derive_http_url(&self.host, &self.port)
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

#[allow(clippy::from_over_into)]
impl Into<SocketAddr> for FuelNodeConfig {
    fn into(self) -> SocketAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
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
        password: String,
        host: String,
        port: String,
        database: String,
    },
}

impl MutableConfig for DatabaseConfig {
    fn inject_opt_env_vars(&mut self) -> Result<()> {
        match self {
            DatabaseConfig::Postgres {
                user,
                password,
                host,
                port,
                database,
            } => {
                if is_opt_env_var(user) {
                    *user = std::env::var(trim_opt_env_key(user))
                        .expect("Failed to read POSTGRES_USER from env.");
                }
                if is_opt_env_var(password) {
                    *password = std::env::var(trim_opt_env_key(password))
                        .expect("Failed to read POSTGRES_PASSWORD from env.");
                }

                if is_opt_env_var(host) {
                    *host = std::env::var(trim_opt_env_key(host))
                        .expect("Failed to read POSTGRES_HOST from env.");
                }

                if is_opt_env_var(port) {
                    *port = std::env::var(trim_opt_env_key(port))
                        .expect("Failed to read POSTGRES_PORT from env.");
                }

                if is_opt_env_var(database) {
                    *database = std::env::var(trim_opt_env_key(database))
                        .expect("Failed to read POSTGRES_DATABASE from env.");
                }
            }
            DatabaseConfig::Sqlite { path } => {
                let os_str = path
                    .as_os_str()
                    .to_str()
                    .expect("Failed to convert path to &str slice");
                if is_opt_env_var(os_str) {
                    *path = PathBuf::from(
                        std::env::var(trim_opt_env_key(os_str)).unwrap_or_else(|_| {
                            format!("Failed to read '{}' from env", os_str)
                        }),
                    );
                }
            }
        }
        Ok(())
    }

    fn derive_socket_addr(&self) -> SocketAddr {
        match self {
            DatabaseConfig::Postgres { host, port, .. } => derive_socket_addr(host, port),
            _ => {
                panic!(
                    "Cannot use MutableConfig::derive_socket_addr on a SQLite database."
                )
            }
        }
    }

    fn derive_http_url(&self) -> String {
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
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    user, password, host, port, database
                )
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
            password: defaults::POSTGRES_PASSWORD.into(),
            host: defaults::POSTGRES_HOST.into(),
            port: defaults::POSTGRES_PORT.into(),
            database: defaults::POSTGRES_DATABASE.into(),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct GraphQLConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: String,
    #[serde(default)]
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
            .expect("Failed to parse GraphQL host.")
    }
}

impl MutableConfig for GraphQLConfig {
    fn inject_opt_env_vars(&mut self) -> Result<()> {
        if is_opt_env_var(&self.host) {
            self.host = std::env::var(trim_opt_env_key(&self.host))
                .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.host));
        }

        if is_opt_env_var(&self.port) {
            self.port = std::env::var(trim_opt_env_key(&self.port))
                .unwrap_or_else(|_| panic!("Failed to read '{}' from env", &self.port));
        }

        Ok(())
    }

    fn derive_socket_addr(&self) -> SocketAddr {
        derive_socket_addr(&self.host, &self.port)
    }

    fn derive_http_url(&self) -> String {
        derive_http_url(&self.host, &self.port)
    }
}

#[derive(Clone, Deserialize, Default, Debug)]
pub struct IndexerConfig {
    #[serde(default)]
    pub fuel_node: FuelNodeConfig,
    #[serde(default)]
    pub graphql_api: GraphQLConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    pub metrics: bool,
}

impl IndexerConfig {
    // Construct a config from args passed to the program. Even if the opt is not passed
    // it could exist as an environment variable, thus the use of `env_or_default`
    pub fn from_opts(args: IndexerArgs) -> IndexerConfig {
        let database = match args.database.as_str() {
            "postgres" => DatabaseConfig::Postgres {
                user: args.postgres_user.unwrap_or_else(|| {
                    env_or_default(
                        EnvVar::PostgresUser,
                        defaults::POSTGRES_USER.to_string(),
                    )
                }),
                password: args.postgres_password.unwrap_or_else(|| {
                    env_or_default(
                        EnvVar::PostgresPassword,
                        defaults::POSTGRES_PASSWORD.to_string(),
                    )
                }),
                host: args.postgres_host.unwrap_or_else(|| {
                    env_or_default(
                        EnvVar::PostgresHost,
                        defaults::POSTGRES_HOST.to_string(),
                    )
                }),
                port: args.postgres_port.unwrap_or_else(|| {
                    env_or_default(
                        EnvVar::PostgresPort,
                        defaults::POSTGRES_PORT.to_string(),
                    )
                }),
                database: args.postgres_database.unwrap_or_else(|| {
                    env_or_default(
                        EnvVar::PostgresDatabase,
                        defaults::POSTGRES_DATABASE.to_string(),
                    )
                }),
            },
            "sqlite" => DatabaseConfig::Sqlite {
                path: args.sqlite_database,
            },
            _ => {
                panic!("Unrecognized database type in options.");
            }
        };

        let mut config = IndexerConfig {
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
            metrics: args.metrics.unwrap_or(false),
        };

        config.inject_opt_env_vars();

        config
    }

    // When building the config via a file, if any section (e.g., graphql, fuel_node, etc),
    // or if any individual setting in a section (e.g., fuel_node.host) is empty, replace it
    // with its respective default value
    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;

        let mut config = IndexerConfig::default();

        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;

        let fuel_config_key = serde_yaml::Value::String("fuel_node".into());
        let graphql_config_key = serde_yaml::Value::String("graphql".into());
        let database_config_key = serde_yaml::Value::String("database".into());

        if let Some(section) = content.get(fuel_config_key) {
            let fuel_node_host = section.get(&serde_yaml::Value::String("host".into()));

            if let Some(fuel_node_host) = fuel_node_host {
                config.fuel_node.host = fuel_node_host.as_str().unwrap().to_string();
            }
            let fuel_node_port = section.get(&serde_yaml::Value::String("port".into()));

            if let Some(fuel_node_port) = fuel_node_port {
                config.fuel_node.port = fuel_node_port.as_u64().unwrap().to_string();
            }
        }

        if let Some(section) = content.get(graphql_config_key) {
            let graphql_api_host = section.get(&serde_yaml::Value::String("host".into()));
            if let Some(graphql_api_host) = graphql_api_host {
                config.graphql_api.host = graphql_api_host.as_str().unwrap().to_string();
            }

            let graphql_api_port = section.get(&serde_yaml::Value::String("port".into()));
            if let Some(graphql_api_port) = graphql_api_port {
                config.graphql_api.port = graphql_api_port.as_u64().unwrap().to_string();
            }

            let graphql_run_migrations =
                section.get(&serde_yaml::Value::String("run_migrations".into()));

            if let Some(graphql_run_migrations) = graphql_run_migrations {
                config.graphql_api.run_migrations =
                    Some(graphql_run_migrations.as_bool().unwrap());
            }
        }

        if let Some(section) = content.get(database_config_key) {
            let pg_section = section.get("postgres");
            let sqlite_section = section.get("sqlite");

            if pg_section.is_some() && sqlite_section.is_some() {
                panic!("'database' section of config file can not contain both postgres and sqlite.");
            }

            if let Some(pg_section) = pg_section {
                let mut pg_user = defaults::POSTGRES_USER.to_string();
                let mut pg_password = defaults::POSTGRES_PASSWORD.to_string();
                let mut pg_host = defaults::POSTGRES_HOST.to_string();
                let mut pg_port = defaults::POSTGRES_PORT.to_string();
                let mut pg_db = defaults::POSTGRES_DATABASE.to_string();

                let pg_host_value =
                    pg_section.get(&serde_yaml::Value::String("host".into()));
                if let Some(pg_host_value) = pg_host_value {
                    pg_host = pg_host_value.as_str().unwrap().to_string();
                }

                let pg_port_value =
                    pg_section.get(&serde_yaml::Value::String("port".into()));
                if let Some(pg_port_value) = pg_port_value {
                    pg_port = pg_port_value.as_u64().unwrap().to_string();
                }

                let pg_username_value =
                    pg_section.get(&serde_yaml::Value::String("user".into()));
                if let Some(pg_username_value) = pg_username_value {
                    pg_user = pg_username_value.as_str().unwrap().to_string();
                }

                let pg_password_value =
                    pg_section.get(&serde_yaml::Value::String("password".into()));
                if let Some(pg_password_value) = pg_password_value {
                    pg_password = pg_password_value.as_str().unwrap().to_string();
                }

                let pg_database_value =
                    pg_section.get(&serde_yaml::Value::String("database".into()));
                if let Some(pg_database_value) = pg_database_value {
                    pg_db = pg_database_value.as_str().unwrap().to_string();
                }

                config.database = DatabaseConfig::Postgres {
                    user: pg_user,
                    password: pg_password,
                    host: pg_host,
                    port: pg_port,
                    database: pg_db,
                };
            }

            if let Some(sqlite_section) = sqlite_section {
                let mut db_path = defaults::SQLITE_DATABASE.to_string();

                let db_path_value =
                    sqlite_section.get(&serde_yaml::Value::String("path".into()));
                if let Some(db_path_value) = db_path_value {
                    db_path = db_path_value.as_str().unwrap().to_string();
                }

                config.database = DatabaseConfig::Sqlite {
                    path: PathBuf::from(db_path),
                };
            }
        }

        config.inject_opt_env_vars();

        Ok(config)
    }

    // Inject env vars into each section of the config
    pub fn inject_opt_env_vars(&mut self) {
        let _ = self.fuel_node.inject_opt_env_vars();
        let _ = self.database.inject_opt_env_vars();
        let _ = self.graphql_api.inject_opt_env_vars();
    }
}

#[cfg(test)]
mod tests {

    use super::DatabaseConfig;
    use super::*;
    use std::fs;

    #[test]
    fn test_indexer_config_will_supplement_entire_config_sections() {
        let config_str = r#"
    ## Fuel Node configuration
    #
    fuel_node:
      host: 1.1.1.1
      port: 9999
    "#;

        let tmp_file_path = "./foo.yaml";

        fs::write(tmp_file_path, config_str).expect("Unable to write file");
        let config = IndexerConfig::from_file(Path::new(tmp_file_path)).unwrap();

        assert_eq!(config.fuel_node.host, "1.1.1.1".to_string());
        assert_eq!(config.fuel_node.port, "9999".to_string());
        assert_eq!(config.graphql_api.host, "127.0.0.1".to_string());

        fs::remove_file(tmp_file_path).unwrap();
    }

    #[test]
    fn test_indexer_config_will_supplement_individual_config_vars_in_sections() {
        let config_str = r#"
        ## Database configuration options. Use either the Postgres
        ## configuration or the SQLite configuration, but not both
        #
        database:
          postgres:
            user: jimmy
            database: my_fancy_db
            password: super_secret_password
    
        "#;

        let tmp_file_path = "./bar.yaml";

        fs::write(tmp_file_path, config_str).expect("Unable to write file");
        let config = IndexerConfig::from_file(Path::new(tmp_file_path)).unwrap();

        assert_eq!(config.fuel_node.host, "127.0.0.1".to_string());
        assert_eq!(config.fuel_node.port, "4000".to_string());
        assert_eq!(config.graphql_api.host, "127.0.0.1".to_string());

        match config.database {
            DatabaseConfig::Postgres {
                user,
                password,
                database,
                ..
            } => {
                assert_eq!(user, "jimmy".to_string());
                assert_eq!(database, "my_fancy_db".to_string());
                assert_eq!(password, "super_secret_password".to_string());

                fs::remove_file(tmp_file_path).unwrap();
            }
            _ => panic!("Incorrect DB type."),
        }
    }
}
