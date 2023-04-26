use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifySignatureRequest {
    pub signature: String,
    pub message: String,
}

#[derive(Serialize)]
pub(crate) struct QueryResponse {
    pub data: Value,
}
