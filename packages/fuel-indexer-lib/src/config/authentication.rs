use crate::{
    config::{IndexerConfigResult, MutConfig},
    defaults,
    defaults::AuthStrategy,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use serde::Deserialize;

const AUTH_ENABLED_KEY: &str = "AUTH_ENABLED";
const AUTH_STRATEGY_KEY: &str = "AUTH_STRATEGY";

#[derive(Clone, Deserialize, Debug)]
pub struct AuthenticationConfig {
    #[serde(default)]
    pub auth_enabled: bool,
    #[serde(default)]
    pub strategy: AuthStrategy,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            auth_enabled: defaults::AUTH_ENABLED,
            strategy: AuthStrategy::Jwt,
        }
    }
}

impl MutConfig for AuthenticationConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        if is_opt_env_var(AUTH_ENABLED_KEY) {
            self.auth_enabled = std::env::var(trim_opt_env_key(AUTH_ENABLED_KEY))?
                .parse()
                .unwrap();
        }

        if is_opt_env_var(AUTH_STRATEGY_KEY) {
            self.strategy = std::env::var(trim_opt_env_key(AUTH_STRATEGY_KEY))?
                .parse()
                .unwrap();
        }

        Ok(())
    }
}
