extern crate alloc;
use crate::sql_types::ColumnType;
use core::convert::TryInto;
use fuel_indexer_types::{
    Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, Jsonb, MessageId, Salt,
};
use serde::{Deserialize, Serialize};

pub use fuel_indexer_database_types as sql_types;

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");
pub const JOIN_DIRECTIVE_NAME: &str = "foreign_key";
pub const UNIQUE_DIRECTIVE_NAME: &str = "unique";

#[cfg(feature = "db-models")]
pub mod db;

pub mod directives;
pub mod utils;

const MAX_STRING_LEN: usize = 255;

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
    Jsonb(Jsonb),
    MessageId(MessageId),
    String255(String),
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
            ColumnType::Jsonb => FtColumn::Jsonb(Jsonb(
                String::from_utf8_lossy(&bytes[..size]).to_string(),
            )),
            ColumnType::MessageId => {
                let message_id =
                    MessageId::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::MessageId(message_id)
            }
            ColumnType::String255 => {
                let trimmed: Vec<u8> = bytes[..size]
                    .iter()
                    .filter_map(|x| if *x != b' ' { Some(*x) } else { None })
                    .collect();

                let s = String::from_utf8_lossy(&trimmed).to_string();

                assert!(s.len() <= MAX_STRING_LEN, "String255 exceeds max length.");
                FtColumn::String255(s)
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
            FtColumn::Jsonb(value) => {
                format!("'{}'", value.0)
            }
            FtColumn::MessageId(value) => {
                format!("'{:x}'", value)
            }
            FtColumn::String255(value) => {
                format!("'{}'", value)
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
        let string255 = FtColumn::String255(String::from("hello world"));
        let jsonb = FtColumn::Jsonb(Jsonb(r#"{"hello":"world"}"#.to_string()));

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
        insta::assert_yaml_snapshot!(string255.query_fragment());
        insta::assert_yaml_snapshot!(jsonb.query_fragment())
    }
}
