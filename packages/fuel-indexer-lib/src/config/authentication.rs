use crate::{
    config::{IndexerConfigResult, MutConfig},
    defaults,
    defaults::AuthScheme,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use serde::Deserialize;

const AUTH_ENABLED_KEY: &str = "auth_enabled";
const AUTH_SCHEME_KEY: &str = "auth_scheme";

#[derive(Clone, Deserialize, Debug)]
pub struct AuthenticationConfig {
    #[serde(default)]
    pub auth_enabled: bool,
    #[serde(default)]
    pub auth_scheme: AuthScheme,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            auth_enabled: defaults::AUTH_ENABLED,
            auth_scheme: AuthScheme::Jwt,
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

        if is_opt_env_var(AUTH_SCHEME_KEY) {
            self.auth_scheme = std::env::var(trim_opt_env_key(AUTH_SCHEME_KEY))?
                .parse()
                .unwrap();
        }

        Ok(())
    }
}
