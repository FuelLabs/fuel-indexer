use crate::{
    scalar::Bytes32, transaction::TransactionData, type_id, TypeId, FUEL_TYPES_NAMESPACE,
};
use serde::{Deserialize, Serialize};

/// Block header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/286
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

/// Fuel indexer-specific block.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Block {
    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/286
    pub height: u64,
    pub id: Bytes32,
    pub header: Header,
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub transactions: Vec<TransactionData>,
}

impl TypeId for Block {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Block") as usize
    }
}
