use crate::{
    config::{derive_http_url, Env, IndexerConfigResult},
    defaults,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use http::Uri;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Clone, Deserialize, Debug)]
pub struct FuelNodeConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: String,
}

impl FuelNodeConfig {
    pub fn health_check_uri(self) -> Uri {
        let base = Uri::from(self);
        format!("{}{}", base, "health")
            .parse()
            .unwrap_or_else(|e| panic!("Failed to build health Uri: {e}"))
    }
}

impl Env for FuelNodeConfig {
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

impl From<FuelNodeConfig> for Uri {
    fn from(config: FuelNodeConfig) -> Self {
        let uri = derive_http_url(&config.host, &config.port);
        uri.parse().unwrap_or_else(|e| {
            panic!("Cannot parse HTTP URI from Fuel node config {config:?}: {e}")
        })
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
