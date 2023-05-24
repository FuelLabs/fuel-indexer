// TODO: https://github.com/FuelLabs/fuel-indexer/issues/286

use crate::{
    scalar::{Bytes32, Signature},
    transaction::TransactionData,
    type_id, TypeId, FUEL_TYPES_NAMESPACE,
};
use serde::{Deserialize, Serialize};

/// Fuel indexer-specific `Header`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderData {
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

/// Fuel indexer-specific `Block`.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockData {
    pub height: u64,
    pub id: Bytes32,
    pub header: HeaderData,
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

/// Fuel indexer-specific `Genesis`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genesis {
    pub chain_config_hash: Bytes32,
    pub coins_root: Bytes32,
    pub contracts_root: Bytes32,
    pub messages_root: Bytes32,
}

/// Fuel indexer-specific `PoA`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoA {
    pub signature: Signature,
}

/// Fuel indexer-specific `Consensus`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Consensus {
    Genesis(Genesis),
    PoA(PoA),
    Unknown,
}
