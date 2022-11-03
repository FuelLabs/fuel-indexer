pub mod log;
pub mod native;
pub mod tx;

pub use fuel_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, MessageId, Salt, Word,
};
pub use fuels_core::types::Bits256;
use serde::{Deserialize, Serialize};

pub type ID = u64;
pub type Int4 = i32;
pub type Int8 = i64;
pub type UInt4 = u32;
pub type UInt8 = u64;
pub type Timestamp = u64;

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Jsonb(pub String);

pub struct IndexMetadata {
    pub id: Bytes32,
    pub block_height: u64,
    pub time: u64,
}

impl IndexMetadata {
    pub fn graphql_schema_fragment() -> &'static str {
        r#"

type IndexMetadataEntity {
    id: Bytes32! @unique
    block_height: UInt8!
    time: Int8!
}
"#
    }
}
