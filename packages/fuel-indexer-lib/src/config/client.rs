use crate::{
    config::{utils::derive_http_url, Env, IndexerConfigResult},
    defaults,
};
pub use clap::Parser;
use http::Uri;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Fuel GraphQL API configuration.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct FuelClientConfig {
    /// Host of the running Fuel node.
    #[serde(default)]
    pub host: String,

    /// Listening port of the running Fuel node.
    #[serde(default)]
    pub port: String,
}

impl FuelClientConfig {
    pub fn health_check_uri(self) -> Uri {
        let base = Uri::from(self);
        format!("{}{}", base, "health")
            .parse()
            .unwrap_or_else(|e| panic!("Failed to build health Uri: {e}"))
    }
}

impl Env for FuelClientConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        Ok(())
    }
}

impl From<FuelClientConfig> for Uri {
    fn from(config: FuelClientConfig) -> Self {
        let uri = derive_http_url(&config.host, &config.port);
        uri.parse().unwrap_or_else(|e| {
            panic!("Cannot parse HTTP URI from Fuel node config {config:?}: {e}")
        })
    }
}

impl Default for FuelClientConfig {
    fn default() -> Self {
        Self {
            host: defaults::FUEL_NODE_HOST.into(),
            port: defaults::FUEL_NODE_PORT.into(),
        }
    }
}

impl From<SocketAddr> for FuelClientConfig {
    fn from(s: SocketAddr) -> FuelClientConfig {
        let parts: Vec<String> = s.to_string().split(':').map(|x| x.to_owned()).collect();
        let host = parts[0].to_owned();
        let port = parts[1].to_owned();
        FuelClientConfig { host, port }
    }
}

impl std::string::ToString for FuelClientConfig {
    fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
