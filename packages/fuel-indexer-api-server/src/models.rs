use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifySignatureRequest {
    pub signature: String,
    pub message: String,
}
