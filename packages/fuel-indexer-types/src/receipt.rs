use crate::{
    scalar::{Address, AssetId, Bytes32, ContractId, MessageId, Nonce},
    TypeId, FUEL_TYPES_NAMESPACE,
};
use fuel_indexer_lib::type_id;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transfer {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}

impl TypeId for Transfer {
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

impl TypeId for Log {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Log") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogData {
    pub contract_id: ContractId,
    pub data: Vec<u8>,
    pub rb: u64,
    pub len: u64,
    pub ptr: u64,
}

impl TypeId for LogData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "LogData") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}

impl TypeId for ScriptResult {
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

impl TypeId for TransferOut {
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
    pub nonce: Nonce,
    pub len: u64,
    pub digest: Bytes32,
    pub data: Vec<u8>,
}

impl TypeId for MessageOut {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "MessageOut") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Return {
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}

impl TypeId for Return {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Return") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Call {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub gas: u64,
    pub fn_name: String,
}

impl TypeId for Call {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Call") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Panic {
    pub contract_id: ContractId,
    pub reason: u32,
}

impl TypeId for Panic {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Panic") as usize
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Revert {
    pub contract_id: ContractId,
    pub error_val: u64,
}

impl TypeId for Revert {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Revert") as usize
    }
}
