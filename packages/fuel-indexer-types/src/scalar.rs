use bytes::Bytes as _Bytes;
pub use fuel_types::{
    Address, AssetId, BlockHeight, Bytes32, Bytes4, Bytes64, Bytes8, ContractId,
    MessageId, Nonce, Salt, Word,
};
use fuels::types::SizedAsciiString;
use serde::{Deserialize, Serialize};

/// Scalar for 32-byte unique ID payloads.
pub type UID = SizedAsciiString<64>;

/// Scalar for object IDs.
pub type ID = UID;

/// Scalar for 4-byte signed integers.
pub type I32 = i32;

/// Scalar for 8-byte signed integers.
pub type I64 = i64;

/// Scalar for 16-byte signed integers.
pub type I128 = i128;

/// Scalar for 4-byte unsigned integers.
pub type U32 = u32;

/// Scalar for 8-byte unsigned integers.
pub type U64 = u64;

/// Scalar for 16-byte unsigned integers.
pub type U128 = u128;

/// Scalar for boolean.
pub type Boolean = bool;

/// Scalar for 1-byte signed integers.
pub type I8 = i8;

/// Scalar for 1-byte unsigned integers.
pub type U8 = u8;

/// Scalar for arbitrarily-sized byte payloads.
pub type Bytes = _Bytes;

/// JSON type used to store arbitrary object payloads.
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Json(pub String);

impl Default for Json {
    fn default() -> Self {
        Json("{}".to_string())
    }
}

impl AsRef<[u8]> for Json {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
