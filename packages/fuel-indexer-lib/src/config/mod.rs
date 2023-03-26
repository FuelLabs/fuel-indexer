pub mod auth;
pub mod database;
pub mod fuel_node;
pub mod graphql;

pub use crate::{
    config::{
        auth::{AuthenticationConfig, AuthenticationStrategy},
        database::DatabaseConfig,
        fuel_node::FuelNodeConfig,
        graphql::GraphQLConfig,
    },
    defaults,
    utils::bool_to_str,
};
pub use clap::{Args, Parser, ValueEnum};
use serde::Deserialize;
use std::{
    fs::File,
    io::Error,
    net::AddrParseError,
    path::{Path, PathBuf},
    str::FromStr,
};
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
    #[strum(serialize = "JWT_SECRET")]
    JwtSecret,
}

pub fn env_or_default(var: EnvVar, default: String) -> String {
    std::env::var(var.as_ref()).unwrap_or(default)
}

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "Indexer Service",
    about = "Standalone binary for the fuel indexer service.",
    version
)]
pub struct IndexerArgs {
    /// Log level passed to the Fuel Indexer service.
    #[clap(long, default_value = "info", value_parser(["info", "debug", "error", "warn"]), help = "Log level passed to the Fuel Indexer service.")]
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

    /// GraphQL API host.
    #[clap(long, help = "GraphQL API host.", default_value = defaults::GRAPHQL_API_HOST)]
    pub graphql_api_host: String,

    /// GraphQL API port.
    #[clap(long, help = "GraphQL API port.", default_value = defaults::GRAPHQL_API_PORT)]
    pub graphql_api_port: String,

    /// Database type.
    #[clap(long, help = "Database type.", default_value = defaults::DATABASE, value_parser(["postgres"]))]
    pub database: String,

    /// Max body size for GraphQL API requests.
    #[clap(long, help = "Max body size for GraphQL API requests.", default_value_t = defaults::MAX_BODY_SIZE )]
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
}

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "Fuel Indexer API Server",
    about = "Fuel indexer GraphQL API",
    version
)]
pub struct ApiServerArgs {
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

    /// GraphQL API host.
    #[clap(long, help = "GraphQL API host.", default_value = defaults::GRAPHQL_API_HOST)]
    pub graphql_api_host: String,

    /// GraphQL API port.
    #[clap(long, help = "GraphQL API port.", default_value = defaults::GRAPHQL_API_PORT)]
    pub graphql_api_port: String,

    /// Database type.
    #[clap(long, help = "Database type.", default_value = defaults::DATABASE, value_parser(["postgres"]))]
    pub database: String,

    /// Max body size for GraphQL API requests.
    #[clap(long, help = "Max body size for GraphQL API requests.", default_value_t = defaults::MAX_BODY_SIZE )]
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
}

fn derive_http_url(host: &String, port: &String) -> String {
    let protocol = match port.as_str() {
        "443" | "4443" => "https",
        _ => "http",
    };

    format!("{protocol}://{host}:{port}")
}

pub trait Env {
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
    pub verbose: bool,
    #[serde(default)]
    pub fuel_node: FuelNodeConfig,
    #[serde(default)]
    pub graphql_api: GraphQLConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    pub metrics: bool,
    pub stop_idle_indexers: bool,
    pub run_migrations: bool,
    pub authentication: AuthenticationConfig,
}

impl From<IndexerArgs> for IndexerConfig {
    fn from(args: IndexerArgs) -> Self {
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
                verbose: bool_to_str(args.verbose),
            },
            _ => {
                panic!("Unrecognized database type in options.");
            }
        };

        let mut config = IndexerConfig {
            verbose: args.verbose,
            database,
            fuel_node: FuelNodeConfig {
                host: args.fuel_node_host,
                port: args.fuel_node_port,
            },
            graphql_api: GraphQLConfig {
                host: args.graphql_api_host,
                port: args.graphql_api_port,
                max_body_size: args.max_body_size,
            },
            metrics: args.metrics,
            stop_idle_indexers: args.stop_idle_indexers,
            run_migrations: args.run_migrations,
            authentication: AuthenticationConfig {
                enabled: args.auth_enabled,
                strategy: args
                    .auth_strategy
                    .map(|x| AuthenticationStrategy::from_str(&x).unwrap()),
                jwt_secret: args.jwt_secret,
                jwt_issuer: args.jwt_issuer,
                jwt_expiry: args.jwt_expiry,
            },
        };

        config
            .inject_opt_env_vars()
            .expect("Failed to inject env vars.");

        config
    }
}

impl From<ApiServerArgs> for IndexerConfig {
    fn from(args: ApiServerArgs) -> Self {
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
                verbose: bool_to_str(args.verbose),
            },
            _ => {
                panic!("Unrecognized database type in options.");
            }
        };

        let mut config = IndexerConfig {
            verbose: args.verbose,
            database,
            fuel_node: FuelNodeConfig {
                host: args.fuel_node_host,
                port: args.fuel_node_port,
            },
            graphql_api: GraphQLConfig {
                host: args.graphql_api_host,
                port: args.graphql_api_port,
                max_body_size: args.max_body_size,
            },
            metrics: args.metrics,
            stop_idle_indexers: defaults::STOP_IDLE_INDEXERS,
            run_migrations: args.run_migrations,
            authentication: AuthenticationConfig {
                enabled: args.auth_enabled,
                strategy: args
                    .auth_strategy
                    .map(|x| AuthenticationStrategy::from_str(&x).unwrap()),
                jwt_secret: args.jwt_secret,
                jwt_issuer: args.jwt_issuer,
                jwt_expiry: args.jwt_expiry,
            },
        };

        config
            .inject_opt_env_vars()
            .expect("Failed to inject env vars.");

        config
    }
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
                verbose: bool_to_str(args.verbose),
            },
            _ => {
                panic!("Unrecognized database type in options.");
            }
        };

        let mut config = IndexerConfig {
            verbose: args.verbose,
            database,
            fuel_node: FuelNodeConfig {
                host: args.fuel_node_host,
                port: args.fuel_node_port,
            },
            graphql_api: GraphQLConfig {
                host: args.graphql_api_host,
                port: args.graphql_api_port,
                max_body_size: args.max_body_size,
            },
            metrics: args.metrics,
            stop_idle_indexers: args.stop_idle_indexers,
            run_migrations: args.run_migrations,
            authentication: AuthenticationConfig {
                enabled: args.auth_enabled,
                strategy: args
                    .auth_strategy
                    .map(|x| AuthenticationStrategy::from_str(&x).unwrap()),
                jwt_secret: args.jwt_secret,
                jwt_issuer: args.jwt_issuer,
                jwt_expiry: args.jwt_expiry,
            },
        };

        config
            .inject_opt_env_vars()
            .expect("Failed to inject env vars.");

        config
    }

    // When building the config via a file, if any section (e.g., graphql, fuel_node, etc),
    // or if any individual setting in a section (e.g., fuel_node.host) is empty, replace it
    // with its respective default value.
    pub fn from_file(path: impl AsRef<Path>) -> IndexerConfigResult<Self> {
        let file = File::open(path)?;

        let mut config = IndexerConfig::default();

        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;

        let metrics_key = serde_yaml::Value::String("metrics".into());
        let stop_idle_indexers_key =
            serde_yaml::Value::String("stop_idle_indexers".into());
        let run_migrations_key = serde_yaml::Value::String("run_migrations".into());
        let verbose_key = serde_yaml::Value::String("verbose".into());

        if let Some(metrics) = content.get(metrics_key) {
            config.metrics = metrics.as_bool().unwrap();
        }

        if let Some(stop_idle_indexers) = content.get(stop_idle_indexers_key) {
            config.stop_idle_indexers = stop_idle_indexers.as_bool().unwrap();
        }

        if let Some(run_migrations) = content.get(run_migrations_key) {
            config.run_migrations = run_migrations.as_bool().unwrap();
        }

        if let Some(verbose) = content.get(verbose_key) {
            config.verbose = verbose.as_bool().unwrap();
        }

        let fuel_config_key = serde_yaml::Value::String("fuel_node".into());
        let graphql_config_key = serde_yaml::Value::String("graphql".into());
        let database_config_key = serde_yaml::Value::String("database".into());
        let auth_config_key = serde_yaml::Value::String("authentication".into());

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

            let max_body_size =
                section.get(&serde_yaml::Value::String("max_body_size".into()));

            if let Some(max_body_size) = max_body_size {
                config.graphql_api.max_body_size =
                    max_body_size.as_u64().unwrap() as usize;
            }
        }

        if let Some(section) = content.get(database_config_key) {
            let pg_section = section.get("postgres");

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
                    verbose: bool_to_str(config.verbose),
                };
            }
        }

        if let Some(section) = content.get(auth_config_key) {
            let auth_enabled =
                section.get(&serde_yaml::Value::String("auth_enabled".into()));
            if let Some(auth_enabled) = auth_enabled {
                config.authentication.enabled = auth_enabled.as_bool().unwrap();
            }

            let strategy =
                section.get(&serde_yaml::Value::String("auth_strategy".into()));
            if let Some(strategy) = strategy {
                config.authentication.strategy = Some(
                    AuthenticationStrategy::from_str(strategy.as_str().unwrap()).unwrap(),
                );
            }

            let jwt_secret = section.get(&serde_yaml::Value::String("jwt_secret".into()));
            if let Some(jwt_secret) = jwt_secret {
                config.authentication.jwt_secret =
                    Some(jwt_secret.as_str().unwrap().to_string());
            }

            let jwt_issuer = section.get(&serde_yaml::Value::String("jwt_issuer".into()));
            if let Some(jwt_issuer) = jwt_issuer {
                config.authentication.jwt_issuer =
                    Some(jwt_issuer.as_str().unwrap().to_string());
            }
        }

        config.inject_opt_env_vars()?;

        Ok(config)
    }

    // Inject env vars into each section of the config
    pub fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        self.fuel_node.inject_opt_env_vars()?;
        self.database.inject_opt_env_vars()?;
        self.graphql_api.inject_opt_env_vars()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::DatabaseConfig;
    use super::*;
    use std::fs;

    const FILE: &str = "foo.yaml";

    #[test]
    fn test_indexer_config_will_supplement_top_level_config_vars() {
        let config_str = r#"
    stop_idle_indexers: true

    ## Fuel Node configuration
    #
    fuel_node:
      host: 1.1.1.1
      port: 9999
    "#;

        fs::write(FILE, config_str).unwrap();
        let config = IndexerConfig::from_file(FILE).unwrap();

        assert!(config.stop_idle_indexers);
        assert_eq!(config.run_migrations, false);
        assert_eq!(config.verbose, false);

        let DatabaseConfig::Postgres { verbose, .. } = config.database;
        assert_eq!(verbose.as_str(), "false");

        fs::remove_file(FILE).unwrap();
    }

    #[test]
    fn test_indexer_config_will_supplement_entire_config_sections() {
        let config_str = r#"
    ## Fuel Node configuration
    #
    fuel_node:
      host: 1.1.1.1
      port: 9999
    "#;

        fs::write(FILE, config_str).unwrap();
        let config = IndexerConfig::from_file(FILE).unwrap();

        assert_eq!(config.fuel_node.host, "1.1.1.1".to_string());
        assert_eq!(config.fuel_node.port, "9999".to_string());
        assert_eq!(config.graphql_api.host, "localhost".to_string());

        fs::remove_file(FILE).unwrap();
    }

    #[test]
    fn test_indexer_config_will_supplement_individual_config_vars_in_sections() {
        let config_str = r#"
        ## Database configuration options.
        #
        database:
          postgres:
            user: jimmy
            database: my_fancy_db
            password: super_secret_password

        "#;

        fs::write(FILE, config_str).unwrap();
        let config = IndexerConfig::from_file(FILE).unwrap();

        assert_eq!(config.fuel_node.host, "localhost".to_string());
        assert_eq!(config.fuel_node.port, "4000".to_string());
        assert_eq!(config.graphql_api.host, "localhost".to_string());

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

                fs::remove_file(FILE).unwrap();
            }
        }
    }
}
