pub mod abi;
pub mod ffi;
pub mod graphql;
pub mod tx;

pub use crate::abi::*;
pub use crate::tx::*;
pub use fuel_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, MessageId, Salt, Word,
};
pub use fuels_core::types::Bits256;
use serde::{Deserialize, Serialize};

pub type Error = Box<dyn std::error::Error>;
pub type ID = u64;
pub type Int4 = i32;
pub type Int8 = i64;
pub type UInt4 = u32;
pub type UInt8 = u64;
pub type Timestamp = u64;
pub type Charfield = String;
pub type Boolean = bool;

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Json(pub String);
