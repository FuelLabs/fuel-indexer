// ** All types in this file need to eventually be replace **
//
// TODO: https://github.com/FuelLabs/fuel-indexer/issues/286

use crate::{
    scalar::{Address, AssetId, Bytes32, ContractId, HexString, Nonce, Signature},
    type_id, TypeId, FUEL_TYPES_NAMESPACE,
};
use chrono::{DateTime, Utc};
pub use fuel_tx::{
    field::{
        BytecodeLength, BytecodeWitnessIndex, GasLimit, GasPrice, Inputs, Maturity,
        Outputs, ReceiptsRoot, Salt as TxFieldSalt, Script, ScriptData, StorageSlots,
        TxPointer as FieldTxPointer, Witnesses,
    },
    Receipt, ScriptExecutionResult, Transaction, TxId, UtxoId, Witness,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransactionData {
    pub transaction: Transaction,
    pub status: TransactionStatusData,
    pub receipts: Vec<Receipt>,
    pub id: TxId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub id: Bytes32,
    pub da_height: u64,
    pub transactions_count: u64,
    pub output_messages_count: u64,
    pub transactions_root: Bytes32,
    pub output_messages_root: Bytes32,
    pub height: u64,
    pub prev_root: Bytes32,
    pub time: i64,
    pub application_hash: Bytes32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockData {
    pub height: u64,
    pub id: Bytes32,
    pub header: Header,
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub consensus: Consensus,
    pub transactions: Vec<TransactionData>,
}

impl TypeId for BlockData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "BlockData") as usize
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    Coin(InputCoin),
    Contract(InputContract),
    Message(InputMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxPointer {
    pub block_height: u32,
    pub tx_index: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputCoin {
    pub utxo_id: UtxoId,
    pub owner: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub tx_pointer: TxPointer,
    pub witness_index: u8,
    pub maturity: u32,
    pub predicate: HexString,
    pub predicate_data: HexString,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputContract {
    pub utxo_id: UtxoId,
    pub balance_root: Bytes32,
    pub state_root: Bytes32,
    pub tx_pointer: TxPointer,
    pub contract_id: ContractId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputMessage {
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Nonce,
    pub witness_index: u8,
    pub data: HexString,
    pub predicate: HexString,
    pub predicate_data: HexString,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionStatusData {
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

impl Default for TransactionStatusData {
    fn default() -> Self {
        TransactionStatusData::SqueezedOut {
            reason: "squeezed out".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractIdFragment {
    pub id: ContractId,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum Output {
    CoinOutput(CoinOutput),
    ContractOutput(ContractOutput),
    ChangeOutput(ChangeOutput),
    VariableOutput(VariableOutput),
    ContractCreated(ContractCreated),
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractOutput {
    pub input_index: i32,
    pub balance_root: Bytes32,
    pub state_root: Bytes32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariableOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractCreated {
    pub contract: ContractIdFragment,
    pub state_root: Bytes32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genesis {
    pub chain_config_hash: Bytes32,
    pub coins_root: Bytes32,
    pub contracts_root: Bytes32,
    pub messages_root: Bytes32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoA {
    pub signature: Signature,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum Consensus {
    Genesis(Genesis),
    PoA(PoA),
    #[default]
    Unknown,
}
