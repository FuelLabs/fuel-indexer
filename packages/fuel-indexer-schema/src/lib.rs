#![deny(unused_crate_dependencies)]

extern crate alloc;

use fuel_indexer_types::prelude::fuel::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const QUERY_ROOT: &str = "QueryRoot";

#[cfg(feature = "db-models")]
pub mod db;
pub mod parser;
pub mod utils;

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");
const NULL_VALUE: &str = "NULL";

pub type IndexerSchemaResult<T> = core::result::Result<T, IndexerSchemaError>;

#[derive(Error, Debug)]
pub enum IndexerSchemaError {
    #[error("Generic error")]
    Generic,
    #[error("GraphQL parser error: {0:?}")]
    ParseError(#[from] async_graphql_parser::Error),
    #[error("Could not build schema: {0:?}")]
    SchemaConstructionError(String),
    #[error("Unable to parse join directive: {0:?}")]
    JoinDirectiveError(String),
    #[error("Unable to build schema field and type map: {0:?}")]
    FieldAndTypeConstructionError(String),
    #[error("This TypeKind is unsupported.")]
    UnsupportedTypeKind,
    #[error("List types are unsupported.")]
    ListTypesUnsupported,
    #[error("Inconsistent use of virtual union types. {0:?}")]
    InconsistentVirtualUnion(String),
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Hash)]
pub enum FtColumn {
    Address(Option<Address>),
    AssetId(Option<AssetId>),
    Blob(Option<Blob>),
    BlockHeight(Option<BlockHeight>),
    Boolean(Option<bool>),
    Bytes32(Option<Bytes32>),
    Bytes4(Option<Bytes4>),
    Bytes64(Option<Bytes64>),
    Bytes8(Option<Bytes8>),
    Charfield(Option<String>),
    ContractId(Option<ContractId>),
    Enum(Option<String>),
    HexString(Option<HexString>),
    ID(Option<UInt8>),
    Identity(Option<Identity>),
    Int1(Option<Int1>),
    Int16(Option<Int16>),
    Int4(Option<Int4>),
    Int8(Option<Int8>),
    Json(Option<Json>),
    MessageId(Option<MessageId>),
    Nonce(Option<Nonce>),
    Salt(Option<Salt>),
    Signature(Option<Signature>),
    Tai64Timestamp(Option<Tai64Timestamp>),
    Timestamp(Option<Int8>),
    TxId(Option<TxId>),
    UInt1(Option<UInt1>),
    UInt16(Option<UInt16>),
    UInt4(Option<UInt4>),
    UInt8(Option<UInt8>),
    Virtual(Option<Virtual>),
    BlockId(Option<BlockId>),
}

impl FtColumn {
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
            FtColumn::Bytes32(value) | FtColumn::BlockId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Nonce(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes64(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::TxId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::HexString(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Signature(value) => match value {
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
            FtColumn::Int1(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt1(value) => match value {
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
            FtColumn::UInt4(value) | FtColumn::BlockHeight(value) => match value {
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
            FtColumn::Tai64Timestamp(value) => match value {
                Some(val) => {
                    let x = hex::encode(val.to_bytes());
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::Salt(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Json(value) | FtColumn::Virtual(value) => match value {
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
                Some(blob) => {
                    let x = hex::encode(blob.as_ref());
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::Enum(value) => match value {
                Some(val) => format!("'{val}'"),
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
