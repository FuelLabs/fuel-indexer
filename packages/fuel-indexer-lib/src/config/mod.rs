pub mod auth;
pub mod cli;
pub mod client;
pub mod database;
pub mod limit;
pub mod utils;
pub mod web;

pub use crate::{
    config::{
        auth::{AuthenticationConfig, AuthenticationStrategy},
        cli::{ApiServerArgs, IndexerArgs},
        client::FuelClientConfig,
        database::DatabaseConfig,
        limit::RateLimitConfig,
        web::WebApiConfig,
    },
    defaults,
    utils::*,
};
use serde::Deserialize;
use std::{fs::File, path::Path, str::FromStr};
use strum::{AsRefStr, EnumString};
use thiserror::Error;

/// Set of PostgresQL configuration constants.
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

/// Return the value of an environment variable or a default value.
pub fn env_or_default(var: EnvVar, default: String) -> String {
    std::env::var(var.as_ref()).unwrap_or(default)
}

/// Result type returned by configuration operations.
type IndexerConfigResult<T> = core::result::Result<T, IndexerConfigError>;

/// Error type returned by configuration operations.
#[derive(Error, Debug)]
pub enum IndexerConfigError {
    #[error("Error parsing env variables from config")]
    EnvVarParseError(#[from] std::env::VarError),
    #[error("Error processing file: {0:?}")]
    ConfigFileError(#[from] anyhow::Error),
    #[error("Error processing YAML file: {0:?}")]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error("Error processing URI: {0:?}")]
    InvalidUriError(#[from] http::uri::InvalidUri),
    #[error("URL parser error: {0:?}")]
    ParseError(#[from] url::ParseError),
    #[error("File IO error: {0:?}")]
    FileIoError(#[from] std::io::Error),
}

/// Used to inject environment variables into configuration.
pub trait Env {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()>;
}

impl Default for IndexerArgs {
    fn default() -> Self {
        Self {
            metering_points: defaults::METERING_POINTS,
            log_level: defaults::LOG_LEVEL.to_string(),
            config: None,
            manifest: None,
            fuel_node_host: defaults::FUEL_NODE_HOST.to_string(),
            fuel_node_port: defaults::FUEL_NODE_PORT.to_string(),
            web_api_host: defaults::WEB_API_HOST.to_string(),
            web_api_port: defaults::WEB_API_PORT.to_string(),
            database: defaults::DATABASE.to_string(),
            max_body_size: defaults::MAX_BODY_SIZE,
            postgres_user: Some(defaults::POSTGRES_USER.to_string()),
            postgres_database: Some(defaults::POSTGRES_DATABASE.to_string()),
            postgres_password: None,
            postgres_host: Some(defaults::POSTGRES_HOST.to_string()),
            postgres_port: Some(defaults::POSTGRES_PORT.to_string()),
            run_migrations: defaults::RUN_MIGRATIONS,
            metrics: defaults::USE_METRICS,
            stop_idle_indexers: defaults::STOP_IDLE_INDEXERS,
            embedded_database: defaults::EMBEDDED_DATABASE,
            auth_enabled: defaults::AUTH_ENABLED,
            auth_strategy: None,
            jwt_secret: None,
            jwt_issuer: None,
            jwt_expiry: None,
            verbose: defaults::VERBOSE_LOGGING,
            local_fuel_node: defaults::LOCAL_FUEL_NODE,
            indexer_net_config: defaults::INDEXER_NET_CONFIG,
            rate_limit: defaults::RATE_LIMIT_ENABLED,
            rate_limit_request_count: Some(defaults::RATE_LIMIT_REQUEST_COUNT),
            rate_limit_window_size: Some(defaults::RATE_LIMIT_WINDOW_SIZE),
            replace_indexer: defaults::REPLACE_INDEXER,
            remove_data: defaults::REMOVE_DATA,
            accept_sql_queries: defaults::ACCEPT_SQL,
            block_page_size: defaults::NODE_BLOCK_PAGE_SIZE,
            enable_blockstore: defaults::ENABLE_BLOCKSTORE,
        }
    }
}

/// Fuel indexer service configuration.
#[derive(Clone, Deserialize, Debug)]
pub struct IndexerConfig {
    pub metering_points: Option<u64>,
    pub log_level: String,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub local_fuel_node: bool,
    #[serde(default)]
    pub indexer_net_config: bool,
    #[serde(default)]
    pub fuel_node: FuelClientConfig,
    #[serde(default)]
    pub web_api: WebApiConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    pub metrics: bool,
    pub stop_idle_indexers: bool,
    pub run_migrations: bool,
    pub authentication: AuthenticationConfig,
    pub rate_limit: RateLimitConfig,
    pub replace_indexer: bool,
    pub accept_sql_queries: bool,
    pub node_block_page_size: usize,
    pub enable_blockstore: bool,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            metering_points: Some(defaults::METERING_POINTS),
            log_level: defaults::LOG_LEVEL.to_string(),
            verbose: defaults::VERBOSE_LOGGING,
            local_fuel_node: defaults::LOCAL_FUEL_NODE,
            indexer_net_config: defaults::INDEXER_NET_CONFIG,
            fuel_node: FuelClientConfig::default(),
            web_api: WebApiConfig::default(),
            database: DatabaseConfig::default(),
            metrics: defaults::USE_METRICS,
            stop_idle_indexers: defaults::STOP_IDLE_INDEXERS,
            run_migrations: defaults::RUN_MIGRATIONS,
            authentication: AuthenticationConfig::default(),
            rate_limit: RateLimitConfig::default(),
            replace_indexer: defaults::REPLACE_INDEXER,
            accept_sql_queries: defaults::ACCEPT_SQL,
            node_block_page_size: defaults::NODE_BLOCK_PAGE_SIZE,
            enable_blockstore: defaults::ENABLE_BLOCKSTORE,
        }
    }
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
                verbose: args.verbose.to_string(),
            },
            _ => {
                panic!("Unrecognized database type in options.");
            }
        };

        let mut config = IndexerConfig {
            metering_points: Some(args.metering_points),
            log_level: args.log_level,
            verbose: args.verbose,
            local_fuel_node: args.local_fuel_node,
            indexer_net_config: args.indexer_net_config,
            database,
            fuel_node: FuelClientConfig {
                host: args.fuel_node_host,
                port: args.fuel_node_port,
            },
            web_api: WebApiConfig {
                host: args.web_api_host,
                port: args.web_api_port,
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
            rate_limit: RateLimitConfig {
                enabled: args.rate_limit,
                request_count: args.rate_limit_request_count,
                window_size: args.rate_limit_window_size,
            },
            replace_indexer: args.replace_indexer,
            accept_sql_queries: args.accept_sql_queries,
            node_block_page_size: args.block_page_size,
            enable_blockstore: args.enable_blockstore,
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
                verbose: args.verbose.to_string(),
            },
            _ => {
                panic!("Unrecognized database type in options.");
            }
        };

        let mut config = IndexerConfig {
            metering_points: Some(defaults::METERING_POINTS),
            log_level: args.log_level,
            verbose: args.verbose,
            local_fuel_node: defaults::LOCAL_FUEL_NODE,
            indexer_net_config: defaults::INDEXER_NET_CONFIG,
            database,
            fuel_node: FuelClientConfig {
                host: args.fuel_node_host,
                port: args.fuel_node_port,
            },
            web_api: WebApiConfig {
                host: args.web_api_host,
                port: args.web_api_port,
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
            rate_limit: RateLimitConfig {
                enabled: args.rate_limit,
                request_count: args.rate_limit_request_count,
                window_size: args.rate_limit_window_size,
            },
            replace_indexer: defaults::REPLACE_INDEXER,
            accept_sql_queries: args.accept_sql_queries,
            node_block_page_size: defaults::NODE_BLOCK_PAGE_SIZE,
            enable_blockstore: defaults::ENABLE_BLOCKSTORE,
        };

        config
            .inject_opt_env_vars()
            .expect("Failed to inject env vars.");

        config
    }
}

impl IndexerConfig {
    // When building the config via a file, if any section (e.g., graphql, fuel_node, etc),
    // or if any individual setting in a section (e.g., fuel_node.host) is empty, replace it
    // with its respective default value.
    pub fn from_file(path: impl AsRef<Path>) -> IndexerConfigResult<Self> {
        let file = File::open(path)?;

        let mut config = IndexerConfig::default();

        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;

        let log_level_key = serde_yaml::Value::String("log_level".into());
        let replace_indexer_key = serde_yaml::Value::String("replace_indexer".into());
        let metering_points_key = serde_yaml::Value::String("metering_points".into());
        let metrics_key = serde_yaml::Value::String("metrics".into());
        let stop_idle_indexers_key =
            serde_yaml::Value::String("stop_idle_indexers".into());
        let run_migrations_key = serde_yaml::Value::String("run_migrations".into());
        let verbose_key = serde_yaml::Value::String("verbose".into());
        let local_fuel_node_key = serde_yaml::Value::String("local_fuel_node".into());
        let indexer_net_config_key =
            serde_yaml::Value::String("indexer_net_config".into());

        let accept_sql_config_key =
            serde_yaml::Value::String("accept_sql_queries".into());

        let node_block_page_size_key =
            serde_yaml::Value::String("block_page_size".into());

        if let Some(accept_sql_queries) = content.get(accept_sql_config_key) {
            config.accept_sql_queries = accept_sql_queries.as_bool().unwrap();
        }

        if let Some(replace_indexer) = content.get(replace_indexer_key) {
            config.replace_indexer = replace_indexer.as_bool().unwrap();
        }

        if let Some(metering_points) = content.get(metering_points_key) {
            config.metering_points = Some(metering_points.as_u64().unwrap());
        }

        if let Some(log_level) = content.get(log_level_key) {
            config.log_level = log_level.as_str().unwrap().to_string();
        }

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

        if let Some(local_fuel_node) = content.get(local_fuel_node_key) {
            config.local_fuel_node = local_fuel_node.as_bool().unwrap();
        }

        if let Some(indexer_net_config) = content.get(indexer_net_config_key) {
            config.indexer_net_config = indexer_net_config.as_bool().unwrap();
        }

        if let Some(node_block_page_size) = content.get(node_block_page_size_key) {
            config.node_block_page_size = node_block_page_size.as_u64().unwrap() as usize;
        }

        let fuel_config_key = serde_yaml::Value::String("fuel_node".into());
        let web_config_key = serde_yaml::Value::String("web_api".into());
        let database_config_key = serde_yaml::Value::String("database".into());
        let auth_config_key = serde_yaml::Value::String("authentication".into());
        let rate_limit_config_key = serde_yaml::Value::String("rate_limit".into());

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

        if let Some(section) = content.get(web_config_key) {
            let web_api_host = section.get(&serde_yaml::Value::String("host".into()));
            if let Some(web_api_host) = web_api_host {
                config.web_api.host = web_api_host.as_str().unwrap().to_string();
            }

            let web_api_port = section.get(&serde_yaml::Value::String("port".into()));
            if let Some(web_api_port) = web_api_port {
                config.web_api.port = web_api_port.as_u64().unwrap().to_string();
            }

            let max_body_size =
                section.get(&serde_yaml::Value::String("max_body_size".into()));

            if let Some(max_body_size) = max_body_size {
                config.web_api.max_body_size = max_body_size.as_u64().unwrap() as usize;
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
                    verbose: config.verbose.to_string(),
                };
            }
        }

        if let Some(section) = content.get(auth_config_key) {
            let auth_enabled = section.get(&serde_yaml::Value::String("enabled".into()));
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

        if let Some(section) = content.get(rate_limit_config_key) {
            let limit_enabled = section.get(&serde_yaml::Value::String("enabled".into()));
            if let Some(limit_enabled) = limit_enabled {
                config.rate_limit.enabled = limit_enabled.as_bool().unwrap();
            }

            let request_count =
                section.get(&serde_yaml::Value::String("request_count".into()));
            if let Some(request_count) = request_count {
                config.rate_limit.request_count = Some(request_count.as_u64().unwrap());
            }

            let window_size =
                section.get(&serde_yaml::Value::String("window_size".into()));
            if let Some(window_size) = window_size {
                config.rate_limit.window_size = Some(window_size.as_u64().unwrap());
            }
        }

        config.inject_opt_env_vars()?;

        Ok(config)
    }

    // Inject env vars into each section of the config
    pub fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        self.fuel_node.inject_opt_env_vars()?;
        self.database.inject_opt_env_vars()?;
        self.web_api.inject_opt_env_vars()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::DatabaseConfig;
    use super::*;
    use std::fs;

    #[test]
    fn test_indexer_config_will_supplement_top_level_config_vars() {
        let file_path: &str = "foo1.yaml";
        let config_str = r#"
    stop_idle_indexers: true

    ## Fuel Node configuration
    #
    fuel_node:
      host: 1.1.1.1
      port: 9999
    "#;

        fs::write(file_path, config_str).unwrap();
        let config = IndexerConfig::from_file(file_path).unwrap();

        assert!(config.stop_idle_indexers);
        assert!(config.run_migrations);
        assert!(!config.verbose);

        let DatabaseConfig::Postgres { verbose, .. } = config.database;
        assert_eq!(verbose.as_str(), "false");

        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_indexer_config_will_supplement_entire_config_sections() {
        let file_path: &str = "foo2.yaml";
        let config_str = r#"
    ## Fuel Node configuration
    #
    fuel_node:
      host: 1.1.1.1
      port: 9999
    "#;

        fs::write(file_path, config_str).unwrap();
        let config = IndexerConfig::from_file(file_path).unwrap();

        assert_eq!(config.fuel_node.host, "1.1.1.1".to_string());
        assert_eq!(config.fuel_node.port, "9999".to_string());
        assert_eq!(config.web_api.host, "localhost".to_string());

        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_indexer_config_will_supplement_individual_config_vars_in_sections() {
        let file_path: &str = "foo3.yaml";
        let config_str = r#"
        ## Database configuration options.
        #
        database:
          postgres:
            user: jimmy
            database: my_fancy_db
            password: super_secret_password

        "#;

        fs::write(file_path, config_str).unwrap();
        let config = IndexerConfig::from_file(file_path).unwrap();

        assert_eq!(config.fuel_node.host, "localhost".to_string());
        assert_eq!(config.fuel_node.port, "4000".to_string());
        assert_eq!(config.web_api.host, "localhost".to_string());

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

                fs::remove_file(file_path).unwrap();
            }
        }
    }
}
