use crate::{
    config::{Env, IndexerConfigResult},
    defaults,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Formatter},
    str::FromStr,
};
use strum::{AsRefStr, EnumString};

const JWT_SECRET_KEY: &str = "JWT_SECRET";
const JWT_ISSUER_KEY: &str = "JWT_ISSUER";

/// Indexer service authentication configuration.
#[derive(Clone, Deserialize, Serialize)]
pub struct AuthenticationConfig {
    /// Require users to authenticate for some operations.
    pub enabled: bool,

    /// Authentication scheme used.
    #[serde(default)]
    pub strategy: Option<AuthenticationStrategy>,

    /// Secret used for JWT scheme (if JWT scheme is specified).
    pub jwt_secret: Option<String>,

    /// Issuer of JWT claims (if JWT scheme is specified).
    pub jwt_issuer: Option<String>,

    /// Amount of time (seconds) before expiring token (if JWT scheme is specified).
    pub jwt_expiry: Option<usize>,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::AUTH_ENABLED,
            strategy: Some(
                AuthenticationStrategy::from_str(defaults::AUTH_STRATEGY)
                    .expect("Invalid auth strategy."),
            ),
            jwt_secret: Some(defaults::JWT_SECRET.to_string()),
            jwt_issuer: Some(defaults::JWT_ISSUER.to_string()),
            jwt_expiry: Some(defaults::JWT_EXPIRY_SECS),
        }
    }
}

impl Debug for AuthenticationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let AuthenticationConfig {
            enabled,
            jwt_expiry,
            strategy,
            jwt_issuer,
            ..
        } = self;
        let _ = f
            .debug_struct("AuthenticationConfig")
            .field("enabled", &enabled)
            .field("strategy", &strategy)
            .field("jwt_secret", &"XXXX")
            .field("jwt_issuer", &jwt_issuer)
            .field("jwt_expiry", &jwt_expiry)
            .finish();

        Ok(())
    }
}

impl Env for AuthenticationConfig {
    fn inject_opt_env_vars(&mut self) -> IndexerConfigResult<()> {
        if is_opt_env_var(JWT_SECRET_KEY) {
            self.strategy = Some(
                std::env::var(trim_opt_env_key(JWT_SECRET_KEY))?
                    .parse()
                    .unwrap(),
            );
        }

        if is_opt_env_var(JWT_ISSUER_KEY) {
            self.jwt_issuer = Some(
                std::env::var(trim_opt_env_key(JWT_ISSUER_KEY))?
                    .parse()
                    .unwrap(),
            );
        }

        Ok(())
    }
}

/// List of authentication strategies.
#[derive(Serialize, Deserialize, EnumString, AsRefStr, Clone, Debug, Eq, PartialEq)]
pub enum AuthenticationStrategy {
    #[strum(ascii_case_insensitive)]
    JWT,
}
