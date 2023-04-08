use crate::{
    config::{Env, IndexerConfigResult},
    defaults,
    utils::{is_opt_env_var, trim_opt_env_key},
};
pub use clap::Parser;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

const JWT_SECRET_KEY: &str = "JWT_SECRET";
const JWT_ISSUER_KEY: &str = "JWT_ISSUER";

/// Indexer service authentication configuration.
#[derive(Clone, Deserialize, Debug)]
pub struct AuthenticationConfig {
    pub enabled: bool,
    #[serde(default)]
    pub strategy: Option<AuthenticationStrategy>,
    pub jwt_secret: Option<String>,
    pub jwt_issuer: Option<String>,
    pub jwt_expiry: Option<usize>,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::AUTH_ENABLED,
            strategy: None,
            jwt_secret: None,
            jwt_issuer: None,
            jwt_expiry: None,
        }
    }
}

impl Env for AuthenticationConfig {
    /// Inject environment variables into `AuthenticationConfig`.
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

/// List of autnetication strategies.
#[derive(Serialize, Deserialize, EnumString, AsRefStr, Clone, Debug, Eq, PartialEq)]
pub enum AuthenticationStrategy {
    #[strum(ascii_case_insensitive)]
    JWT,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Claims {
    /// Subject (to whom token refers).
    pub sub: String,
    /// Issuer.
    pub iss: String,
    /// Issued at (as UTC timestamp).
    pub iat: usize,
    /// Expiration time (as UTC timestamp).
    pub exp: usize,
}

/// JWT authentication claims.
///
/// The payload of the JWT token if JWt authentication is enabled.
/// Note that `Claims` are _only_ used by JWT authentication.
impl Claims {
    /// Like `Claims::new`, but with `iat` and `exp` values that indicate
    /// the claims have yet to be authenticated.
    pub fn unauthenticated() -> Self {
        Self {
            sub: "".to_string(),
            iss: "".to_string(),
            iat: 1,
            exp: 1,
        }
    }

    /// Whether or not the given set of claims have been authenticated.
    pub fn is_unauthenticated(&self) -> bool {
        self.exp == 1 && self.iat == 1
    }
}
