use std::str::FromStr;

use crate::{
    config::{Env, IndexerConfigResult},
    defaults,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use http::Uri;
use serde::Deserialize;

use super::IndexerConfigError;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseConfig {
    Postgres {
        user: String,
        password: String,
        host: String,
        port: String,
        database: String,
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
            } => {
                format!("postgres://{user}:{password}@{host}:{port}/{database}")
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

impl From<DatabaseConfig> for Uri {
    fn from(_: DatabaseConfig) -> Self {
        unimplemented!()
    }
}

impl FromStr for DatabaseConfig {
    type Err = IndexerConfigError;

    fn from_str(db_url: &str) -> Result<Self, Self::Err> {
        let scheme_and_conn_details: Vec<&str> = db_url.split(r"://").collect();

        let (scheme, conn_details) =
            (scheme_and_conn_details[0], scheme_and_conn_details[1]);

        match scheme.to_lowercase().as_str() {
            "postgres" => {
                let conn_str_and_database: Vec<&str> = conn_details.split('/').collect();

                let conn_str: Vec<&str> = conn_str_and_database[0].split('@').collect();
                let credentials: Vec<&str> = conn_str[0].split(':').collect();

                let user = credentials[0];
                let password = if credentials.len() == 2 {
                    credentials[1]
                } else {
                    ""
                };

                let host_str: Vec<&str> = conn_str[1].split(':').collect();
                let (host, port) = (host_str[0], host_str[1]);

                let database = if conn_str_and_database.len() == 2 {
                    conn_str_and_database[1]
                } else {
                    ""
                };

                Ok(DatabaseConfig::Postgres {
                    user: user.to_string(),
                    password: password.to_string(),
                    host: host.to_string(),
                    port: port.to_string(),
                    database: database.to_string(),
                })
            }
            _ => Err(IndexerConfigError::DbUrlParseError),
        }
    }
}
