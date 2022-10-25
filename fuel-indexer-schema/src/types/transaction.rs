use crate::types::Jsonb;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

// NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
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
                r#"{{"status":"failed","block":"{block_id}","time":"{time}","reason":"{reason}"}}"#
            )),
            TransactionStatus::Submitted { submitted_at } => Jsonb(format!(
                r#"{{"status":"submitted","time":"{submitted_at}"}}"#
            )),
            TransactionStatus::Success { block_id, time } => Jsonb(format!(
                r#"{{"status":"success","block":"{block_id}","time":"{time}"}}"#
            )),
        }
    }
}
