use crate::types::Jsonb;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

// NOTE: Temporarily wrapping fuel_gql_client::client::types::TransactionStatus because
// using just fuel_gql_client::client::types::TransactionStatus requires importing the
// entire fuel_gql_client crate, which won't easily compile to WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Failure {
        block_id: String,
        time: DateTime<Utc>,
        reason: String,
    },
    Submitted {
        submitted_at: DateTime<Utc>,
    },
    Success {
        block_id: String,
        time: DateTime<Utc>,
    },
}

impl Default for TransactionStatus {
    fn default() -> Self {
        Self::Success {
            block_id: "0".into(),
            time: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
        }
    }
}

impl From<TransactionStatus> for Jsonb {
    fn from(t: TransactionStatus) -> Jsonb {
        match t {
            TransactionStatus::Failure {
                block_id,
                time,
                reason,
            } => Jsonb(format!(
                "FAILED | Block({}) | {} | {}",
                block_id, time, reason
            )),
            TransactionStatus::Submitted { submitted_at } => {
                Jsonb(format!("SUBMITTED | {}", submitted_at))
            }
            TransactionStatus::Success { block_id, time } => {
                Jsonb(format!("SUCCESS | Block({}) | {}", block_id, time))
            }
        }
    }
}
