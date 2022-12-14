use crate::{
    config::{IndexerConfigResult, MutableConfig},
    defaults,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use http::Uri;
use serde::Deserialize;
use std::net::SocketAddr;

use super::derive_http_url;

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

// impl From<Uri> for GraphQLConfig {
//     fn from(uri: Uri) -> Self {
//         GraphQLConfig {
//             host: uri.host().expect("Uri has no host.").to_string(),
//             port: uri.port().expect("Uri has no port.").to_string(),
//             run_migrations: None,
//         }
//     }
// }

impl From<GraphQLConfig> for Uri {
    fn from(config: GraphQLConfig) -> Self {
        let uri = derive_http_url(&config.host, &config.port);
        uri.parse().unwrap_or_else(|e| {
            panic!(
                "Failed to derive Uri from GraphQL config: {:?}: {}",
                config, e,
            )
        })
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
            .unwrap_or_else(|e| panic!("Failed to parse GraphQL host.: {}", e))
    }
}

impl MutableConfig for GraphQLConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        if is_opt_env_var(&self.host) {
            self.host = std::env::var(trim_opt_env_key(&self.host))?;
        }

        if is_opt_env_var(&self.port) {
            self.port = std::env::var(trim_opt_env_key(&self.port))?;
        }

        Ok(())
    }
}
