use crate::scalar::Json;
use chrono::{DateTime, NaiveDateTime, Utc};
pub use fuel_tx::{
    field::{
        BytecodeLength, BytecodeWitnessIndex, GasLimit, GasPrice, Inputs, Maturity,
        Outputs, ReceiptsRoot, Salt as TxFieldSalt, Script, ScriptData, StorageSlots,
        TxPointer, Witnesses,
    },
    Receipt, ScriptExecutionResult, Transaction, TxId, UtxoId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TransactionData {
    pub transaction: Transaction,
    pub status: TransactionStatus,
    pub receipts: Vec<Receipt>,
    pub id: TxId,
}

// NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Failure {
        block_id: String,
        time: DateTime<Utc>,
        reason: String,
    },
    SqueezedOut {
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
            time: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0)
                    .expect("Failed to create timestamp"),
                Utc,
            ),
        }
    }
}

impl From<TransactionStatus> for Json {
    fn from(t: TransactionStatus) -> Json {
        match t {
            TransactionStatus::Failure {
                block_id,
                time,
                reason,
            } => Json(format!(
                r#"{{"status":"failed","block":"{block_id}","time":"{time}","reason":"{reason}"}}"#
            )),
            TransactionStatus::SqueezedOut { reason } => Json(format!(
                r#"{{"status":"squeezed_out","reason":"{reason}"}}"#
            )),
            TransactionStatus::Submitted { submitted_at } => Json(format!(
                r#"{{"status":"submitted","time":"{submitted_at}"}}"#
            )),
            TransactionStatus::Success { block_id, time } => Json(format!(
                r#"{{"status":"success","block":"{block_id}","time":"{time}"}}"#
            )),
        }
    }
}
