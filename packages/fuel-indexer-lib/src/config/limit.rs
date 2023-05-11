use crate::{
    config::{Env, IndexerConfigResult},
    defaults,
};
pub use clap::Parser;
use serde::Deserialize;

/// Indexer service rate limit configuration.
#[derive(Clone, Deserialize, Debug)]
pub struct RateLimitConfig {
    #[serde(default)]
    /// Enable rate limiting.
    pub enabled: bool,

    /// Maximum number of requests to allow over --rate-limit-window..
    pub request_count: Option<u64>,

    /// Number of seconds over which to allow --rate-limit-rps.
    pub window_size: Option<u64>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::RATE_LIMIT_ENABLED,
            request_count: None,
            window_size: None,
        }
    }
}

impl Env for RateLimitConfig {
    /// Inject environment variables into `RateLimitConfig`.
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        Ok(())
    }
}
