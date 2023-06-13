use bytes::Bytes;
pub use fuel_tx::{
    Address, AssetId, Bytes32, Bytes4, Bytes64, Bytes8, ContractId, MessageId, Salt, Word,
};
pub use fuel_types::{BlockHeight, Nonce};
pub use fuels::{
    core::try_from_bytes,
    types::{
        bech32::{Bech32Address, Bech32ContractId},
        Bits256, Identity, SizedAsciiString,
    },
};
use serde::{Deserialize, Serialize};
use tai64::Tai64;

/// Scalar for object IDs.
pub type ID = u64;

/// Scalar for 4-byte signed integers.
pub type Int4 = i32;

/// Scalar for 8-byte signed integers.
pub type Int8 = i64;

/// Scalar for 16-byte signed integers.
pub type Int16 = i128;

/// Scalar for 4-byte unsigned integers.
pub type UInt4 = u32;

/// Scalar for 8-byte unsigned integers.
pub type UInt8 = u64;

/// Scalar for 16-byte unsigned integers.
pub type UInt16 = u128;

/// Scalar for 8-byte integers aliased as `Timestamp`s.
pub type Timestamp = u64;

/// Scalar for arbitrarily sized `String`s aliased as `Charfield`s.
pub type Charfield = String;

/// Scalar for boolean.
pub type Boolean = bool;

/// Scalar for 64-byte signature payloads.
pub type Signature = Bytes64;

/// Scalar for arbitrarily sized byte payloads aliased as `HexString`.
pub type HexString = Bytes;

/// Scalar for `Tai64` timestamps aliased as `Tai64Timestamp`.
pub type Tai64Timestamp = Tai64;

/// Scalar for 32-byte payloads aliased as `BlockId`.
pub type BlockId = Bytes32;

/// Scalar for 1-byte signed integers.
pub type Int1 = i8;

/// Scalar for 1-byte unsigned integers.
pub type UInt1 = u8;

/// Blob type used to store arbitrarily sized UTF-8 payloads.
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub struct Blob(pub Vec<u8>);

impl From<Vec<u8>> for Blob {
    fn from(value: Vec<u8>) -> Self {
        Blob(value)
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Blob> for Vec<u8> {
    fn from(value: Blob) -> Self {
        value.0
    }
}

/// JSON type used to store types tagged with a `@norelation` directive in
/// GraphQL schema. Aliased as `NoRelation`.
pub type NoRelation = Json;

/// JSON type used to store arbitrary object payloads.
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Json(pub String);

impl Default for Json {
    fn default() -> Self {
        Json("{}".to_string())
    }
}

macro_rules! json_impl {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Json {
                fn from(value: $ty) -> Self {
                    Json(value.to_string())
                }
            }
        )*
    }
}

macro_rules! blob_impl {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Blob {
                fn from(value: $ty) -> Self {
                    Blob::from(bincode::serialize(&value).unwrap().to_vec())
                }
            }
        )*
    }
}

json_impl!(i32, i64, i128, u32, u64, u128);
blob_impl!(i32, i64, i128, u32, u64, u128);
