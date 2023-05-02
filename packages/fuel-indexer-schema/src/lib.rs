#![deny(unused_crate_dependencies)]

extern crate alloc;
use crate::sql_types::ColumnType;
use core::convert::TryInto;
use fuel_indexer_types::{
    try_from_bytes, Address, AssetId, Blob, Bytes32, Bytes4, Bytes8, ContractId,
    Identity, Int16, Int4, Int8, Json, MessageId, Salt, UInt16, UInt4, UInt8,
};
use serde::{Deserialize, Serialize};

pub use fuel_indexer_database_types as sql_types;

#[cfg(feature = "db-models")]
pub mod db;
pub mod utils;

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");
pub const UNIQUE_DIRECTIVE_NAME: &str = "unique";
const MAX_CHARFIELD_LENGTH: usize = 255;
const NULL_VALUE: &str = "NULL";

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Hash)]
pub enum FtColumn {
    ID(Option<UInt8>),
    Address(Option<Address>),
    AssetId(Option<AssetId>),
    Bytes4(Option<Bytes4>),
    Bytes8(Option<Bytes8>),
    Bytes32(Option<Bytes32>),
    ContractId(Option<ContractId>),
    Int4(Option<Int4>),
    Int8(Option<Int8>),
    Int16(Option<Int16>),
    UInt4(Option<UInt4>),
    UInt8(Option<UInt8>),
    UInt16(Option<UInt16>),
    Timestamp(Option<Int8>),
    Salt(Option<Salt>),
    Json(Option<Json>),
    MessageId(Option<MessageId>),
    Charfield(Option<String>),
    Identity(Option<Identity>),
    Boolean(Option<bool>),
    Blob(Option<Blob>),
}

impl FtColumn {
    pub fn new(ty: ColumnType, size: usize, bytes: &[u8]) -> FtColumn {
        match ty {
            ColumnType::ID => {
                let ident = u64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::ID(Some(ident))
            }
            ColumnType::Address => {
                let address =
                    Address::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Address(Some(address))
            }
            ColumnType::AssetId => {
                let asset_id =
                    AssetId::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::AssetId(Some(asset_id))
            }
            ColumnType::Bytes4 => {
                let bytes =
                    Bytes4::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Bytes4(Some(bytes))
            }
            ColumnType::Bytes8 => {
                let bytes =
                    Bytes8::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Bytes8(Some(bytes))
            }
            ColumnType::Bytes32 => {
                let bytes =
                    Bytes32::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Bytes32(Some(bytes))
            }
            ColumnType::ContractId => {
                let contract_id =
                    ContractId::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::ContractId(Some(contract_id))
            }
            ColumnType::Salt => {
                let salt = Salt::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Salt(Some(salt))
            }
            ColumnType::Int4 => {
                let int4 = i32::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Int4(Some(int4))
            }
            ColumnType::Int8 => {
                let int8 = i64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Int8(Some(int8))
            }
            ColumnType::Int16 => {
                let int16 = i128::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Int16(Some(int16))
            }
            ColumnType::UInt4 => {
                let int4 = u32::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::UInt4(Some(int4))
            }
            ColumnType::UInt8 => {
                let int8 = u64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::UInt8(Some(int8))
            }
            ColumnType::UInt16 => {
                let uint16 = u128::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::UInt16(Some(uint16))
            }
            ColumnType::Timestamp => {
                let int8 = i64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Timestamp(Some(int8))
            }
            ColumnType::Blob => {
                FtColumn::Blob(Some(fuel_indexer_types::Blob(bytes[..size].to_vec())))
            }
            ColumnType::ForeignKey => {
                panic!("ForeignKey not supported for FtColumn.");
            }
            ColumnType::Json => FtColumn::Json(Some(Json(
                String::from_utf8_lossy(&bytes[..size]).to_string(),
            ))),
            ColumnType::MessageId => {
                let message_id =
                    MessageId::try_from(&bytes[..size]).expect("Invalid slice length");
                FtColumn::MessageId(Some(message_id))
            }
            ColumnType::Charfield => {
                let s = String::from_utf8_lossy(&bytes[..size]).to_string();

                assert!(
                    s.len() <= MAX_CHARFIELD_LENGTH,
                    "Charfield exceeds max length."
                );
                FtColumn::Charfield(Some(s))
            }
            ColumnType::Identity => {
                let identity: Identity =
                    try_from_bytes(&bytes[..size]).expect("Invalid slice length");
                FtColumn::Identity(Some(identity))
            }
            ColumnType::Boolean => {
                let value = u8::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Boolean(Some(value != 0))
            }
            ColumnType::Object => {
                panic!("Object not supported for FtColumn.");
            }
        }
    }

    pub fn query_fragment(&self) -> String {
        match self {
            FtColumn::ID(value) => {
                if let Some(val) = value {
                    format!("{val}")
                } else {
                    panic!("Schema fields of type ID cannot be nullable")
                }
            }
            FtColumn::Address(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::AssetId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes4(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes8(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes32(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::ContractId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Int4(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Int8(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Int16(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt4(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt8(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt16(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Timestamp(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Salt(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Json(value) => match value {
                Some(val) => {
                    let x = &val.0;
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::MessageId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Charfield(value) => match value {
                Some(val) => format!("'{val}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Identity(value) => match value {
                Some(val) => match val {
                    Identity::Address(v) => format!("'{v:x}'",),
                    Identity::ContractId(v) => format!("'{v:x}'",),
                },
                None => String::from(NULL_VALUE),
            },
            FtColumn::Boolean(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Blob(value) => match value {
                Some(fuel_indexer_types::Blob(val)) => {
                    let x = hex::encode(val);
                    format!("'{}'", x)
                }
                None => String::from(NULL_VALUE),
            },
        }
    }
}

mod tests {
    #[test]
    fn test_fragments_some_types() {
        use super::*;

        let id = FtColumn::ID(Some(123456));
        let addr =
            FtColumn::Address(Some(Address::try_from([0x12; 32]).expect("Bad bytes")));
        let asset_id =
            FtColumn::AssetId(Some(AssetId::try_from([0xA5; 32]).expect("Bad bytes")));
        let bytes4 =
            FtColumn::Bytes4(Some(Bytes4::try_from([0xF0; 4]).expect("Bad bytes")));
        let bytes8 =
            FtColumn::Bytes8(Some(Bytes8::try_from([0x9D; 8]).expect("Bad bytes")));
        let bytes32 =
            FtColumn::Bytes32(Some(Bytes32::try_from([0xEE; 32]).expect("Bad bytes")));
        let contractid = FtColumn::ContractId(Some(
            ContractId::try_from([0x78; 32]).expect("Bad bytes"),
        ));
        let int4 = FtColumn::Int4(Some(i32::from_le_bytes([0x78; 4])));
        let int8 = FtColumn::Int8(Some(i64::from_le_bytes([0x78; 8])));
        let int16 = FtColumn::Int16(Some(i128::from_le_bytes([0x78; 16])));
        let uint4 = FtColumn::UInt4(Some(u32::from_le_bytes([0x78; 4])));
        let uint8 = FtColumn::UInt8(Some(u64::from_le_bytes([0x78; 8])));
        let uint16 = FtColumn::UInt16(Some(u128::from_le_bytes([0x78; 16])));
        let int64 = FtColumn::Timestamp(Some(i64::from_le_bytes([0x78; 8])));
        let salt = FtColumn::Salt(Some(Salt::try_from([0x31; 32]).expect("Bad bytes")));
        let message_id = FtColumn::MessageId(Some(
            MessageId::try_from([0x0F; 32]).expect("Bad bytes"),
        ));
        let charfield = FtColumn::Charfield(Some(String::from("hello world")));
        let json = FtColumn::Json(Some(Json(r#"{"hello":"world"}"#.to_string())));
        let identity = FtColumn::Identity(Some(Identity::Address(
            Address::try_from([0x12; 32]).unwrap(),
        )));

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
        insta::assert_yaml_snapshot!(int16.query_fragment());
        insta::assert_yaml_snapshot!(uint4.query_fragment());
        insta::assert_yaml_snapshot!(uint8.query_fragment());
        insta::assert_yaml_snapshot!(uint16.query_fragment());
        insta::assert_yaml_snapshot!(int64.query_fragment());
        insta::assert_yaml_snapshot!(message_id.query_fragment());
        insta::assert_yaml_snapshot!(charfield.query_fragment());
        insta::assert_yaml_snapshot!(json.query_fragment());
        insta::assert_yaml_snapshot!(identity.query_fragment());
    }

    #[test]
    fn test_fragments_none_types() {
        use super::*;

        let addr_none = FtColumn::Address(None);
        let asset_id_none = FtColumn::AssetId(None);
        let bytes4_none = FtColumn::Bytes4(None);
        let bytes8_none = FtColumn::Bytes8(None);
        let bytes32_none = FtColumn::Bytes32(None);
        let contractid_none = FtColumn::ContractId(None);
        let int4_none = FtColumn::Int4(None);
        let int8_none = FtColumn::Int8(None);
        let int16_none = FtColumn::Int8(None);
        let uint4_none = FtColumn::UInt4(None);
        let uint8_none = FtColumn::UInt8(None);
        let uint16_none = FtColumn::UInt8(None);
        let int64_none = FtColumn::Timestamp(None);
        let salt_none = FtColumn::Salt(None);
        let message_id_none = FtColumn::MessageId(None);
        let charfield_none = FtColumn::Charfield(None);
        let json_none = FtColumn::Json(None);
        let identity_none = FtColumn::Identity(None);

        insta::assert_yaml_snapshot!(addr_none.query_fragment());
        insta::assert_yaml_snapshot!(asset_id_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes4_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes8_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes32_none.query_fragment());
        insta::assert_yaml_snapshot!(contractid_none.query_fragment());
        insta::assert_yaml_snapshot!(salt_none.query_fragment());
        insta::assert_yaml_snapshot!(int4_none.query_fragment());
        insta::assert_yaml_snapshot!(int8_none.query_fragment());
        insta::assert_yaml_snapshot!(int16_none.query_fragment());
        insta::assert_yaml_snapshot!(uint4_none.query_fragment());
        insta::assert_yaml_snapshot!(uint8_none.query_fragment());
        insta::assert_yaml_snapshot!(uint16_none.query_fragment());
        insta::assert_yaml_snapshot!(int64_none.query_fragment());
        insta::assert_yaml_snapshot!(message_id_none.query_fragment());
        insta::assert_yaml_snapshot!(charfield_none.query_fragment());
        insta::assert_yaml_snapshot!(json_none.query_fragment());
        insta::assert_yaml_snapshot!(identity_none.query_fragment());
    }

    #[test]
    #[should_panic(expected = "Schema fields of type ID cannot be nullable")]
    fn test_panic_on_none_id_fragment() {
        use super::*;

        let id_none = FtColumn::ID(None);

        insta::assert_yaml_snapshot!(id_none.query_fragment());
    }
}
