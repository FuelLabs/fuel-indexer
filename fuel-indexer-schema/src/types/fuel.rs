use crate::{types::TransactionStatus, utils::type_id};
pub use fuel_tx::Receipt;
use fuel_tx::{Transaction, TxId};
pub use fuel_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, MessageId, Salt, Word,
};
pub use fuels_core::types::Bits256;
use serde::{Deserialize, Serialize};

pub const FUEL_TYPES_NAMESPACE: &str = "fuel";

pub trait NativeFuelTypeIdent {
    fn path_ident_str() -> &'static str;
    fn type_id() -> usize;
}

// TODO: We could also create ABI JSON files with these native Fuel indexer-macro types <( '.' )>
// These aren't actually schema-related types so they should be moved.
// https://github.com/FuelLabs/fuel-indexer/issues/285
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct B256 {}

impl NativeFuelTypeIdent for B256 {
    fn path_ident_str() -> &'static str {
        "BlockData"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TransactionData {
    pub transaction: Transaction,
    pub status: TransactionStatus,
    pub receipts: Vec<Receipt>,
    pub id: TxId,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockData {
    pub height: u64,
    pub id: Bytes32,
    pub time: i64,
    pub producer: Address,
    pub transactions: Vec<TransactionData>,
}

impl NativeFuelTypeIdent for BlockData {
    fn path_ident_str() -> &'static str {
        "BlockData"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

impl BlockData {
    pub fn macro_attribute_ident_str() -> &'static str {
        "block"
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transfer {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}

impl NativeFuelTypeIdent for Transfer {
    fn path_ident_str() -> &'static str {
        "Transfer"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Log {
    pub contract_id: ContractId,
    pub ra: u64,
    pub rb: u64,
}

impl NativeFuelTypeIdent for Log {
    fn path_ident_str() -> &'static str {
        "Log"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

// NOTE: Keeping for now, but I don't believe we need this.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogData {
    pub contract_id: ContractId,
    pub data: Vec<u8>,
    pub rb: u64,
    pub len: u64,
    pub ptr: u64,
}

impl NativeFuelTypeIdent for LogData {
    fn path_ident_str() -> &'static str {
        "LogData"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}

impl NativeFuelTypeIdent for ScriptResult {
    fn path_ident_str() -> &'static str {
        "ScriptResult"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransferOut {
    pub contract_id: ContractId,
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}

impl NativeFuelTypeIdent for TransferOut {
    fn path_ident_str() -> &'static str {
        "TransferOut"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MessageOut {
    pub message_id: MessageId,
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Bytes32,
    pub len: u64,
    pub digest: Bytes32,
    pub data: Vec<u8>,
}

impl NativeFuelTypeIdent for MessageOut {
    fn path_ident_str() -> &'static str {
        "MessageOut"
    }

    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, Self::path_ident_str()) as usize
    }
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Jsonb(pub String);

pub type ID = u64;
pub type Int4 = i32;
pub type Int8 = i64;
pub type UInt4 = u32;
pub type UInt8 = u64;
pub type Timestamp = u64;
