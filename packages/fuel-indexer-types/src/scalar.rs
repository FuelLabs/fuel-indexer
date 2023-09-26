pub use fuel_types::{
    Address, AssetId, BlockHeight, Bytes32, Bytes4, Bytes64, Bytes8, ContractId,
    MessageId, Nonce, Salt, Word,
};
use fuels::types::SizedAsciiString;
use serde::{Deserialize, Serialize};

/// Scalar for 256-bit value.
pub type B256 = [u8; 32];

/// Scalar for 512-bit unique ID payloads.
pub type UID = SizedAsciiString<64>;

/// Scalar for object IDs.
pub type ID = UID;

/// Scalar for 32-bit signed integers.
pub type I32 = i32;

/// Scalar for 64-bit signed integers.
pub type I64 = i64;

/// Scalar for 128-bit signed integers.
pub type I128 = i128;

/// Scalar for 32-bit unsigned integers.
pub type U32 = u32;

/// Scalar for 64-bit unsigned integers.
pub type U64 = u64;

/// Scalar for 128-bit unsigned integers.
pub type U128 = u128;

/// Scalar for boolean.
pub type Boolean = bool;

/// Scalar for 8-bit signed integers.
pub type I8 = i8;

/// Scalar for 8-bit unsigned integers.
pub type U8 = u8;

/// Scalar for 16-bit signed integers.
pub type U16 = u16;

/// Scalar for 16-bit unsigned integers.
pub type I16 = i16;

/// Scalar for arbitrarily-sized byte payloads.
pub type Bytes = Vec<u8>;

/// JSON type used to store arbitrary object payloads.
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Json(String);

impl Json {
    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn new(s: String) -> Self {
        Json(s)
    }
}

impl AsRef<str> for Json {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

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
