pub mod fuel;
pub mod transaction;

pub use fuels::{
    core::types::{Bits256, SizedAsciiString},
    tx::{Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, MessageId, Salt, Word},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Jsonb(pub String);

pub type ID = u64;
pub type Int4 = i32;
pub type Int8 = i64;
pub type UInt4 = u32;
pub type UInt8 = u64;
pub type Timestamp = u64;
