use crate::{
    config::{utils::derive_http_url, Env, IndexerConfigResult},
    defaults,
    utils::derive_socket_addr,
};
pub use clap::Parser;
use http::Uri;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Clone, Deserialize, Debug)]
pub struct WebApiConfig {
    /// Web API host.
    #[serde(default)]
    pub host: String,

    /// Web API port.
    #[serde(default)]
    pub port: String,

    /// Max body size for web API requests.
    #[serde(default)]
    pub max_body_size: usize,
}

impl std::string::ToString for WebApiConfig {
    fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl From<WebApiConfig> for Uri {
    fn from(c: WebApiConfig) -> Self {
        let uri = derive_http_url(&c.host, &c.port);
        uri.parse().unwrap_or_else(|e| {
            panic!("Cannot parse HTTP URI from web config: {c:?}: {e}")
        })
    }
}

impl Default for WebApiConfig {
    fn default() -> Self {
        Self {
            host: defaults::WEB_API_HOST.into(),
            port: defaults::WEB_API_PORT.into(),
            max_body_size: defaults::MAX_BODY_SIZE,
        }
    }
}

impl From<WebApiConfig> for SocketAddr {
    fn from(cfg: WebApiConfig) -> SocketAddr {
        derive_socket_addr(&cfg.host, &cfg.port)
    }
}

impl Env for WebApiConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        Ok(())
    }
}
