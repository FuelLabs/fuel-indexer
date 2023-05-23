use crate::{
    scalar::Bytes32, transaction::TransactionData, type_id, TypeId, FUEL_TYPES_NAMESPACE,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use fuel_core_client::client::schema::block::{
    Block as ClientBlock, Header as ClientHeader,
};
use serde::{Deserialize, Serialize};

/// Block header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/945
    pub id: Bytes32,
    pub da_height: u64,
    pub transactions_count: u64,
    pub output_messages_count: u64,
    pub transactions_root: Bytes32,
    pub output_messages_root: Bytes32,
    pub height: u64,
    pub prev_root: Bytes32,
    pub time: Option<DateTime<Utc>>,
    pub application_hash: Bytes32,
}

impl From<ClientHeader> for Header {
    fn from(client_header: ClientHeader) -> Self {
        let naive = NaiveDateTime::from_timestamp_opt(client_header.time.0.to_unix(), 0);
        let time = naive.map(|time| DateTime::<Utc>::from_utc(time, Utc));

        Self {
            id: client_header.id.0 .0,
            da_height: client_header.da_height.0,
            transactions_count: client_header.transactions_count.0,
            output_messages_count: client_header.output_messages_count.0,
            transactions_root: client_header.transactions_root.0 .0,
            output_messages_root: client_header.output_messages_root.0 .0,
            height: client_header.height.0,
            prev_root: client_header.prev_root.0 .0,
            time,
            application_hash: client_header.application_hash.0 .0,
        }
    }
}

/// Fuel indexer-specific block.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockData {
    pub height: u64,
    pub id: Bytes32,
    pub header: Header,
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub transactions: Vec<TransactionData>,
}

impl TypeId for BlockData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "BlockData") as usize
    }
}

impl From<ClientBlock> for BlockData {
    fn from(client_block: ClientBlock) -> Self {
        let producer = client_block.block_producer().map(|pk| pk.hash());
        Self {
            id: client_block.id.0 .0,
            height: client_block.header.height.0,
            time: client_block.header.time.0.to_unix(),
            header: client_block.header.into(),
            producer,
            transactions: Vec::new(),
        }
    }
}
