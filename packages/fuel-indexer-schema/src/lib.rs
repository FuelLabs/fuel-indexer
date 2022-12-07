extern crate alloc;
use crate::sql_types::ColumnType;
use core::convert::TryInto;
use fuel_indexer_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, Identity, Json, MessageId,
    Salt,
};
use serde::{Deserialize, Serialize};

pub use fuel_indexer_database_types as sql_types;

#[cfg(feature = "db-models")]
pub mod db;
pub mod directives;
pub mod utils;

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");
pub const UNIQUE_DIRECTIVE_NAME: &str = "unique";
const MAX_CHARFIELD_LENGTH: usize = 255;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Hash)]
pub enum FtColumn {
    ID(u64),
    Address(Address),
    AssetId(AssetId),
    Bytes4(Bytes4),
    Bytes8(Bytes8),
    Bytes32(Bytes32),
    ContractId(ContractId),
    Int4(i32),
    Int8(i64),
    UInt4(u32),
    UInt8(u64),
    Timestamp(i64),
    Salt(Salt),
    Json(Json),
    MessageId(MessageId),
    Charfield(String),
    Identity(Identity),
    Boolean(bool),
}

impl FtColumn {
    pub fn new(ty: ColumnType, size: usize, bytes: &[u8]) -> FtColumn {
        match ty {
            ColumnType::ID => {
                let ident = u64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::ID(ident)
            }
            ColumnType::Address => {
                let address =
                    Address::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Address(address)
            }
            ColumnType::AssetId => {
                let asset_id =
                    AssetId::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::AssetId(asset_id)
            }
            ColumnType::Bytes4 => {
                let bytes =
                    Bytes4::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Bytes4(bytes)
            }
            ColumnType::Bytes8 => {
                let bytes =
                    Bytes8::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Bytes8(bytes)
            }
            ColumnType::Bytes32 => {
                let bytes =
                    Bytes32::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Bytes32(bytes)
            }
            ColumnType::ContractId => {
                let contract_id =
                    ContractId::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::ContractId(contract_id)
            }
            ColumnType::Salt => {
                let salt = Salt::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Salt(salt)
            }
            ColumnType::Int4 => {
                let int4 = i32::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Int4(int4)
            }
            ColumnType::Int8 => {
                let int8 = i64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Int8(int8)
            }
            ColumnType::UInt4 => {
                let int4 = u32::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::UInt4(int4)
            }
            ColumnType::UInt8 => {
                let int8 = u64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::UInt8(int8)
            }
            ColumnType::Timestamp => {
                let int8 = i64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Timestamp(int8)
            }
            ColumnType::Blob => {
                panic!("Blob not supported for FtColumn.");
            }
            ColumnType::ForeignKey => {
                panic!("ForeignKey not supported for FtColumn.");
            }
            ColumnType::Json => {
                FtColumn::Json(Json(String::from_utf8_lossy(&bytes[..size]).to_string()))
            }
            ColumnType::MessageId => {
                let message_id =
                    MessageId::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::MessageId(message_id)
            }
            ColumnType::Charfield => {
                let s = String::from_utf8_lossy(&bytes[..size]).trim().to_string();

                assert!(
                    s.len() <= MAX_CHARFIELD_LENGTH,
                    "Charfield exceeds max length."
                );
                FtColumn::Charfield(s)
            }
            ColumnType::Identity => {
                let identity =
                    Identity::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Identity(identity)
            }
            ColumnType::Boolean => {
                let value = u8::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Boolean(value != 0)
            }
        }
    }

    pub fn query_fragment(&self) -> String {
        match self {
            FtColumn::ID(value) => {
                format!("{}", value)
            }
            FtColumn::Address(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::AssetId(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::Bytes4(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::Bytes8(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::Bytes32(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::ContractId(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::Int4(value) => {
                format!("{}", value)
            }
            FtColumn::Int8(value) => {
                format!("{}", value)
            }
            FtColumn::UInt4(value) => {
                format!("{}", value)
            }
            FtColumn::UInt8(value) => {
                format!("{}", value)
            }
            FtColumn::Timestamp(value) => {
                format!("{}", value)
            }
            FtColumn::Salt(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::Json(value) => {
                format!("'{}'", value.0)
            }
            FtColumn::MessageId(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::Charfield(value) => {
                format!("'{}'", value)
            }
            FtColumn::Identity(value) => match value {
                Identity::Address(v) => format!("'00{:x}'", v),
                Identity::ContractId(v) => format!("'01{:x}'", v),
            },
            FtColumn::Boolean(value) => {
                format!("{}", value)
            }
        }
    }
}

mod tests {
    #[test]
    fn test_fragments() {
        use super::*;

        let id = FtColumn::ID(123456);
        let addr = FtColumn::Address(Address::try_from([0x12; 32]).expect("Bad bytes"));
        let asset_id =
            FtColumn::AssetId(AssetId::try_from([0xA5; 32]).expect("Bad bytes"));
        let bytes4 = FtColumn::Bytes4(Bytes4::try_from([0xF0; 4]).expect("Bad bytes"));
        let bytes8 = FtColumn::Bytes8(Bytes8::try_from([0x9D; 8]).expect("Bad bytes"));
        let bytes32 =
            FtColumn::Bytes32(Bytes32::try_from([0xEE; 32]).expect("Bad bytes"));
        let contractid =
            FtColumn::ContractId(ContractId::try_from([0x78; 32]).expect("Bad bytes"));
        let int4 = FtColumn::Int4(i32::from_le_bytes([0x78; 4]));
        let int8 = FtColumn::Int8(i64::from_le_bytes([0x78; 8]));
        let uint4 = FtColumn::UInt4(u32::from_le_bytes([0x78; 4]));
        let uint8 = FtColumn::UInt8(u64::from_le_bytes([0x78; 8]));
        let int64 = FtColumn::Timestamp(i64::from_le_bytes([0x78; 8]));
        let salt = FtColumn::Salt(Salt::try_from([0x31; 32]).expect("Bad bytes"));
        let message_id =
            FtColumn::MessageId(MessageId::try_from([0x0F; 32]).expect("Bad bytes"));
        let charfield = FtColumn::Charfield(String::from("hello world"));
        let json = FtColumn::Json(Json(r#"{"hello":"world"}"#.to_string()));
        let identity =
            FtColumn::Identity(Identity::Address(Address::try_from([0x12; 32]).unwrap()));

        insta::assert_yaml_snapshot!(id.query_fragment());
        insta::assert_yaml_snapshot!(addr.query_fragment());
        insta::assert_yaml_snapshot!(asset_id.query_fragment());
        insta::assert_yaml_snapshot!(bytes4.query_fragment());
        insta::assert_yaml_snapshot!(bytes8.query_fragment());
        insta::assert_yaml_snapshot!(bytes32.query_fragment());
        insta::assert_yaml_snapshot!(contractid.query_fragment());
        insta::assert_yaml_snapshot!(salt.query_fragment());
        insta::assert_yaml_snapshot!(int4.query_fragment());
        insta::assert_yaml_snapshot!(int8.query_fragment());
        insta::assert_yaml_snapshot!(uint4.query_fragment());
        insta::assert_yaml_snapshot!(uint8.query_fragment());
        insta::assert_yaml_snapshot!(int64.query_fragment());
        insta::assert_yaml_snapshot!(message_id.query_fragment());
        insta::assert_yaml_snapshot!(charfield.query_fragment());
        insta::assert_yaml_snapshot!(json.query_fragment());
        insta::assert_yaml_snapshot!(identity.query_fragment());
    }
}
