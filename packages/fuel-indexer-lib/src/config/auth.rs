use crate::{
    config::{IndexerConfigResult, MutConfig},
    defaults,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

const AUTH_ENABLED_KEY: &str = "AUTH_ENABLED";
const AUTH_STRATEGY_KEY: &str = "AUTH_STRATEGY";
const JWT_SECRET_KEY: &str = "JWT_SECRET";

#[derive(Clone, Deserialize, Debug)]
pub struct AuthenticationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub strategy: AuthStrategy,
    pub jwt_secret: Option<String>,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::AUTH_ENABLED,
            strategy: AuthStrategy::JWT,
            jwt_secret: Some(defaults::JWT_SECRET.to_string()),
        }
    }
}

impl MutConfig for AuthenticationConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        if is_opt_env_var(AUTH_ENABLED_KEY) {
            self.enabled = std::env::var(trim_opt_env_key(AUTH_ENABLED_KEY))?
                .parse()
                .unwrap();
        }

        if is_opt_env_var(AUTH_STRATEGY_KEY) {
            self.strategy = std::env::var(trim_opt_env_key(AUTH_STRATEGY_KEY))?
                .parse()
                .unwrap();
        }

        if is_opt_env_var(JWT_SECRET_KEY) {
            self.strategy = std::env::var(trim_opt_env_key(JWT_SECRET_KEY))?
                .parse()
                .unwrap();
        }

        Ok(())
    }
}

#[derive(
    Serialize, Deserialize, EnumString, AsRefStr, Clone, Debug, Default, Eq, PartialEq,
)]
#[serde(rename_all = "lowercase")]
pub enum AuthStrategy {
    #[default]
    JWT,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (to whom token refers).
    pub sub: String,

    /// Issuer.
    pub iss: String,

    /// Issued at (as UTC timestamp).
    pub iat: usize,

    // Expiration time (as UTC timestamp).
    pub exp: usize,
}
