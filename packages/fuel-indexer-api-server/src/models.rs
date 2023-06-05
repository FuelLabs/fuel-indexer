use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Request for signature verification.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifySignatureRequest {
    pub signature: String,
    pub message: String,
}

/// GraphQL web API response.
#[derive(Serialize)]
pub(crate) struct QueryResponse {
    pub data: Value,
}

/// JWT claims.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Claims {
    /// Subject (to whom token refers).
    sub: String,

    /// Issuer.
    iss: String,

    /// Issued at (as UTC timestamp).
    iat: usize,

    /// Expiration time (as UTC timestamp).
    exp: usize,
}

/// The payload of the JWT token if JWT authentication is enabled.
impl Claims {
    /// Create a new set of claims.
    pub fn new(sub: String, iss: String, expiry: usize) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        Self {
            sub,
            iss,
            iat: now,
            exp: now + expiry,
        }
    }

    /// The subject of the claims.
    pub fn sub(&self) -> &str {
        &self.sub
    }

    /// The issuer of the claims.
    pub fn iss(&self) -> &str {
        &self.iss
    }

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
