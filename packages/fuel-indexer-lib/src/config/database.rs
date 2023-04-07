use crate::{
    config::{Env, IndexerConfigResult},
    defaults,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use http::Uri;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};
use url::{ParseError, Url};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseConfig {
    Postgres {
        user: String,
        password: String,
        host: String,
        port: String,
        database: String,
        verbose: String,
    },
}

impl Env for DatabaseConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        match self {
            DatabaseConfig::Postgres {
                user,
                password,
                host,
                port,
                database,
                ..
            } => {
                if is_opt_env_var(user) {
                    *user = std::env::var(trim_opt_env_key(user))?;
                }
                if is_opt_env_var(password) {
                    *password = std::env::var(trim_opt_env_key(password))?;
                }

                if is_opt_env_var(host) {
                    *host = std::env::var(trim_opt_env_key(host))?;
                }

                if is_opt_env_var(port) {
                    *port = std::env::var(trim_opt_env_key(port))?;
                }

                if is_opt_env_var(database) {
                    *database = std::env::var(trim_opt_env_key(database))?;
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
                verbose,
            } => {
                let params = [("verbose", verbose)]
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<String>>()
                    .join("&");
                format!("postgres://{user}:{password}@{host}:{port}/{database}?{params}")
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
                verbose,
                ..
            } => {
                let _ = f
                    .debug_struct("PostgresConfig")
                    .field("user", &user)
                    .field("password", &"XXXX")
                    .field("host", &host)
                    .field("port", &port)
                    .field("database", &database)
                    .field("verbose", &verbose)
                    .finish();
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
            verbose: defaults::VERBOSE_DB_LOGGING.into(),
        }
    }
}

impl From<DatabaseConfig> for Uri {
    fn from(_: DatabaseConfig) -> Self {
        unimplemented!()
    }
}

impl FromStr for DatabaseConfig {
    type Err = ParseError;

    fn from_str(db_url: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(db_url)?;
        let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
        let value = params.get("verbose").unwrap_or(&"false".into()).to_owned();

        match url.scheme() {
            "postgres" => {
                let user = url.username();
                let password = url.password().unwrap_or_default();
                let host = url.host().ok_or(ParseError::EmptyHost).unwrap();
                let port = url.port().ok_or(ParseError::InvalidPort).unwrap();
                let database = if url.path_segments().is_some() {
                    url.path_segments().unwrap().next().unwrap()
                } else {
                    ""
                };

                Ok(DatabaseConfig::Postgres {
                    user: user.to_string(),
                    password: password.to_string(),
                    host: host.to_string(),
                    port: port.to_string(),
                    database: database.to_string(),
                    verbose: value,
                })
            }
            _ => {
                unimplemented!("Unsupported database. Please check your database URL.")
            }
        }
    }
}
