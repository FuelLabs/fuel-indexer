pub mod abi;
pub mod ffi;
pub mod graphql;
pub mod tx;

pub use crate::abi::*;
pub use crate::tx::*;
pub use fuel_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, MessageId, Salt, Word,
};
pub use fuels_core::types::{Bits256, SizedAsciiString};
pub use fuels_types::bech32::{Bech32Address, Bech32ContractId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type Error = Box<dyn std::error::Error>;
pub type ID = u64;
pub type Int4 = i32;
pub type Int8 = i64;
pub type UInt4 = u32;
pub type UInt8 = u64;
pub type Timestamp = u64;
pub type Charfield = String;
pub type Boolean = bool;
pub type Blob = Vec<u8>;

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Json(pub String);

// IMPORTANT: https://github.com/launchbadge/sqlx/issues/499
pub fn type_id(namespace: &str, type_name: &str) -> i64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(
        &Sha256::digest(format!("{}:{}", namespace, type_name).as_bytes())[..8],
    );
    i64::from_le_bytes(bytes)
}
