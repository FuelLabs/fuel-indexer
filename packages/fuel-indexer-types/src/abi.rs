use crate::{
    tx::{Transaction, TransactionStatus, TxId},
    type_id, Address, AssetId, Bytes32, ContractId, MessageId,
};
pub use fuel_tx::Receipt;
pub use fuels_types::Identity;
use serde::{Deserialize, Serialize};

pub const FUEL_TYPES_NAMESPACE: &str = "fuel";

pub trait NativeFuelType {
    fn type_id() -> usize;
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
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub transactions: Vec<TransactionData>,
}

impl NativeFuelType for BlockData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "BlockData") as usize
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

impl NativeFuelType for Transfer {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Transfer") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Log {
    pub contract_id: ContractId,
    pub ra: u64,
    pub rb: u64,
}

impl NativeFuelType for Log {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Log") as usize
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

impl NativeFuelType for LogData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "LogData") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}

impl NativeFuelType for ScriptResult {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "ScriptResult") as usize
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

impl NativeFuelType for TransferOut {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "TransferOut") as usize
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

impl NativeFuelType for MessageOut {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "MessageOut") as usize
    }
}

pub struct Return {
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}

impl NativeFuelType for Return {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Return") as usize
    }
}
