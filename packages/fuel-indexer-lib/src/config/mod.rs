pub mod database;
pub mod fuel_node;
pub mod graphql;

pub use crate::{
    config::{
        database::DatabaseConfig, fuel_node::FuelNodeConfig, graphql::GraphQLConfig,
    },
    defaults,
};
pub use clap::Parser;
use serde::Deserialize;
use std::fs::File;
use std::io::Error;
use std::net::AddrParseError;
use std::path::{Path, PathBuf};
use strum::{AsRefStr, EnumString};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerConfigError {
    #[error("Invalid address: {0:?}")]
    InvalidSocketAddr(#[from] AddrParseError),
    #[error("Error parsing env variables from config")]
    EnvVarParseError(#[from] std::env::VarError),
    #[error("Error processing file: {0:?}")]
    ConfigFileError(#[from] Error),
    #[error("Error processing YAML file: {0:?}")]
    SerdeYamlError(#[from] serde_yaml::Error),
}

type IndexerConfigResult<T> = core::result::Result<T, IndexerConfigError>;

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
    about = "Standalone binary for the fuel indexer service",
    version
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
    pub sqlite_database: String,
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
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()>;
}

impl std::string::ToString for FuelNodeConfig {
    fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
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
    pub fn from_file(path: &Path) -> IndexerConfigResult<Self> {
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

                config.database = DatabaseConfig::Sqlite { path: db_path };
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
