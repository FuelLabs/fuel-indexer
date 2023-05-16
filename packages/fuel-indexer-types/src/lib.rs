pub mod abi;
pub mod ffi;
pub mod graphql;
pub mod tx;

pub use crate::abi::*;
pub use crate::tx::*;
use bytes::Bytes;
pub use fuel_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes64, Bytes8, ContractId, MessageId, Salt, Word,
};
pub use fuels::{
    core::try_from_bytes,
    types::{
        bech32::{Bech32Address, Bech32ContractId},
        Identity, SizedAsciiString,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tai64::Tai64;

pub type Error = Box<dyn std::error::Error>;
pub type ID = u64;
pub type Int4 = i32;
pub type Int8 = i64;
pub type Int16 = i128;
pub type UInt4 = u32;
pub type UInt8 = u64;
pub type UInt16 = u128;
pub type Timestamp = u64;
pub type Charfield = String;
pub type Boolean = bool;
pub type Signature = Bytes64;
pub type Nonce = Bytes32;
pub type HexString = Bytes;
pub type Tai64Timestamp = Tai64;

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
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

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Json(pub String);

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

// IMPORTANT: https://github.com/launchbadge/sqlx/issues/499
pub fn type_id(namespace: &str, type_name: &str) -> i64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(
        &Sha256::digest(format!("{namespace}:{type_name}").as_bytes())[..8],
    );
    i64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_json_blob_id() {
        let id: ID = 123;
        let as_json: Json = id.into();
        let as_bytes: Blob = id.into();

        assert_eq!(as_json, Json("123".to_string()));
        assert_eq!(as_bytes, Blob(vec![123, 0, 0, 0, 0, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_int4() {
        let int: Int4 = 42;
        let as_json: Json = int.into();
        let as_bytes: Blob = int.into();

        assert_eq!(as_json, Json("42".to_string()));
        assert_eq!(as_bytes, Blob(vec![42, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_int8() {
        let int: Int8 = 1234567890;
        let as_json: Json = int.into();
        let as_bytes: Blob = int.into();

        assert_eq!(as_json, Json("1234567890".to_string()));
        assert_eq!(as_bytes, Blob(vec![210, 2, 150, 73, 0, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_int16() {
        let int: Int16 = 123456789012345;
        let as_json: Json = int.into();
        let as_bytes: Blob = int.into();

        assert_eq!(as_json, Json("123456789012345".to_string()));
        assert_eq!(
            as_bytes,
            Blob(vec![
                121, 223, 13, 134, 72, 112, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ])
        );
    }

    #[test]
    fn test_into_json_blob_uint4() {
        let uint: UInt4 = 7;
        let as_json: Json = uint.into();
        let as_bytes: Blob = uint.into();

        assert_eq!(as_json, Json("7".to_string()));
        assert_eq!(as_bytes, Blob(vec![7, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_uint8() {
        let uint: UInt8 = 1234567890;
        let as_json: Json = uint.into();
        let as_bytes: Blob = uint.into();

        assert_eq!(as_json, Json("1234567890".to_string()));
        assert_eq!(as_bytes, Blob(vec![210, 2, 150, 73, 0, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_uint16() {
        let uint: UInt16 = 123456789012345;
        let as_json: Json = uint.into();
        let as_bytes: Blob = uint.into();

        assert_eq!(as_json, Json("123456789012345".to_string()));
        assert_eq!(
            as_bytes,
            Blob(vec![
                121, 223, 13, 134, 72, 112, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ])
        );
    }

    #[test]
    fn test_into_json_blob_timestamp() {
        let timestamp: Timestamp = 1234567890;
        let as_json: Json = timestamp.into();
        let as_bytes: Blob = timestamp.into();

        assert_eq!(as_json, Json("1234567890".to_string()));
        assert_eq!(as_bytes, Blob(vec![210, 2, 150, 73, 0, 0, 0, 0]));
    }
}
