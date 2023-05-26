// ** All types in this file need to eventually be replace **
//
// All struct names below are prefixed with 'Client' to avoid conflicts with
// GraphQL entities, as well as to make it clear that these need to be relaced
// from fuel_client_schema.
//
// TODO: https://github.com/FuelLabs/fuel-indexer/issues/286

use crate::scalar::{Address, AssetId, Bytes32, ContractId, HexString, Json, Nonce};
use chrono::{DateTime, NaiveDateTime, Utc};
pub use fuel_tx::UtxoId as ClientUtxoId;
use serde::{Deserialize, Serialize};

pub enum ClientInput {
    Coin(ClientInputCoin),
    Contract(ClientInputContract),
    Message(ClientInputMessage),
}

pub struct ClientTxPointer {
    pub block_height: u32,
    pub tx_index: u64,
}

pub struct ClientInputCoin {
    pub utxo_id: ClientUtxoId,
    pub owner: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub tx_pointer: ClientTxPointer,
    pub witness_index: u8,
    pub maturity: u32,
    pub predicate: HexString,
    pub predicate_data: HexString,
}

pub struct ClientInputContract {
    pub utxo_id: ClientUtxoId,
    pub balance_root: Bytes32,
    pub state_root: Bytes32,
    pub tx_pointer: ClientTxPointer,
    pub contract_id: ContractId,
}

pub struct ClientInputMessage {
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Nonce,
    pub witness_index: u8,
    pub data: HexString,
    pub predicate: HexString,
    pub predicate_data: HexString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientTransactionStatusData {
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

impl Default for ClientTransactionStatusData {
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

impl From<ClientTransactionStatusData> for Json {
    fn from(t: ClientTransactionStatusData) -> Json {
        match t {
            ClientTransactionStatusData::Failure {
                block_id,
                time,
                reason,
            } => Json(format!(
                r#"{{"status":"failed","block":"{block_id}","time":"{time}","reason":"{reason}"}}"#
            )),
            ClientTransactionStatusData::SqueezedOut { reason } => Json(format!(
                r#"{{"status":"squeezed_out","reason":"{reason}"}}"#
            )),
            ClientTransactionStatusData::Submitted { submitted_at } => Json(format!(
                r#"{{"status":"submitted","time":"{submitted_at}"}}"#
            )),
            ClientTransactionStatusData::Success { block_id, time } => Json(format!(
                r#"{{"status":"success","block":"{block_id}","time":"{time}"}}"#
            )),
        }
    }
}

pub struct ClientContractIdFragment {
    pub id: ContractId,
}

#[derive(Default)]
pub enum ClientOutput {
    CoinOutput(ClientCoinOutput),
    ContractOutput(ClientContractOutput),
    ChangeOutput(ClientChangeOutput),
    VariableOutput(ClientVariableOutput),
    ContractCreated(ClientContractCreated),
    #[default]
    Unknown,
}

pub struct ClientCoinOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

pub struct ClientContractOutput {
    pub input_index: i32,
    pub balance_root: Bytes32,
    pub state_root: Bytes32,
}

pub struct ClientChangeOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

pub struct ClientVariableOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

pub struct ClientContractCreated {
    pub contract: ClientContractIdFragment,
    pub state_root: Bytes32,
}
