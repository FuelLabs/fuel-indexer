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
pub struct GraphQLConfig {
    /// GraphQL API host.
    #[serde(default)]
    pub host: String,

    /// GraphQL API port.
    #[serde(default)]
    pub port: String,

    /// Max body size for GraphQL API requests.
    #[serde(default)]
    pub max_body_size: usize,
}

impl std::string::ToString for GraphQLConfig {
    fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl From<GraphQLConfig> for Uri {
    fn from(c: GraphQLConfig) -> Self {
        let uri = derive_http_url(&c.host, &c.port);
        uri.parse().unwrap_or_else(|e| {
            panic!("Cannot parse HTTP URI from GraphQL config: {c:?}: {e}")
        })
    }
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            host: defaults::GRAPHQL_API_HOST.into(),
            port: defaults::GRAPHQL_API_PORT.into(),
            max_body_size: defaults::MAX_BODY_SIZE,
        }
    }
}

impl From<GraphQLConfig> for SocketAddr {
    fn from(cfg: GraphQLConfig) -> SocketAddr {
        derive_socket_addr(&cfg.host, &cfg.port)
    }
}

impl Env for GraphQLConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        Ok(())
    }
}
