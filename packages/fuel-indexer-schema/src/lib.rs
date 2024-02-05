//! # fuel_indexer_schema
//!
//! A collection of utilities used to create SQL-based objects that interact with
//! objects being pulled from, and being persisted to, the database backend.

// TODO: Deny `clippy::unused_crate_dependencies` when including feature-flagged dependency `itertools`

extern crate alloc;

use fuel_indexer_lib::MAX_ARRAY_LENGTH;
use fuel_indexer_types::{scalar::*, Identity};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "db-models")]
pub mod db;

pub mod join;

/// Placeholder value for SQL `NULL` values.
const NULL_VALUE: &str = "NULL";

/// Result type used by indexer schema operations.
pub type IndexerSchemaResult<T> = core::result::Result<T, IndexerSchemaError>;

/// Error type used by indexer schema operations.
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

/// `FtColumn` is an abstraction that represents a sized type that can be persisted to, and
/// fetched from the database.
///
/// Each `FtColumn` corresponds to a Fuel-specific GraphQL scalar type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Hash)]
pub enum FtColumn {
    Address(Option<Address>),
    Array(Option<Vec<FtColumn>>),
    AssetId(Option<AssetId>),
    Boolean(Option<bool>),
    Bytes(Option<Bytes>),
    Bytes32(Option<Bytes32>),
    Bytes4(Option<Bytes4>),
    Bytes64(Option<Bytes64>),
    Bytes8(Option<Bytes8>),
    ContractId(Option<ContractId>),
    Enum(Option<String>),
    I128(Option<I128>),
    I16(Option<I16>),
    I32(Option<I32>),
    I64(Option<I64>),
    I8(Option<I8>),
    ID(Option<UID>),
    Identity(Option<Identity>),
    Json(Option<Json>),
    String(Option<String>),
    U128(Option<U128>),
    U16(Option<U16>),
    U32(Option<U32>),
    U64(Option<U64>),
    U8(Option<U8>),
    UID(Option<UID>),
}

impl FtColumn {
    /// Return query fragments for `INSERT` statements.
    ///
    /// Since `FtColumn` column is used when compiling indexers we can panic here. Anything that panics,
    /// will panic when compiling indexers, so will be caught before runtime.
    pub fn query_fragment(&self) -> String {
        match self {
            FtColumn::ID(value) => {
                if let Some(val) = value {
                    format!("'{val}'")
                } else {
                    panic!("Schema fields of type `ID` cannot be nullable.")
                }
            }
            FtColumn::UID(value) => match value {
                Some(val) => format!("'{val}'"),
                None => String::from(NULL_VALUE),
            },
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
            FtColumn::Bytes64(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::ContractId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::I32(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::I8(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::U8(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::I64(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::I16(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::U16(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::I128(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::U32(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::U64(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::U128(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Json(value) => match value {
                Some(val) => {
                    let x = &val.0;
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::String(value) => match value {
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
            FtColumn::Bytes(value) => match value {
                Some(blob) => {
                    let x = hex::encode(blob);
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::Enum(value) => match value {
                Some(val) => format!("'{val}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Array(arr) => match arr {
                Some(arr) => {
                    assert!(
                        arr.len() < MAX_ARRAY_LENGTH,
                        "Array length exceeds maximum allowed length."
                    );

                    // If the array has no items, then we have no `FtColumn`s from which to determine
                    // what type of PostgreSQL array this is. In this case, the user should be using a
                    // inner required (outer optional) array (e.g., [Foo!]) in their schema.
                    //
                    // Ideally we need a way to validate this in something like `fuel_indexer_lib::graphql::GraphQLSchemaValidator`.
                    if arr.is_empty() {
                        return String::from(NULL_VALUE);
                    }

                    let discriminant = std::mem::discriminant(&arr[0]);
                    let result = arr
                            .iter()
                            .map(|e| {
                                if std::mem::discriminant(e) != discriminant {
                                    panic!(
                                        "Array elements are not of the same column type. Expected {discriminant:#?} - Actual: {:#?}",
                                        std::mem::discriminant(e)
                                    )
                                } else {
                                    e.to_owned().query_fragment()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(",");

                    // We have to force sqlx to see this as a JSON type else it will think this type
                    // should be TEXT
                    let suffix = match arr[0] {
                        FtColumn::Json(_) => "::json[]",
                        _ => "",
                    };

                    // Using ARRAY syntax vs curly braces so we can keep the single quotes used by
                    // `ColumnType::query_fragment`
                    format!("ARRAY [{result}]{suffix}")
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

        let uid = FtColumn::ID(Some(
            UID::new(
                "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            )
            .unwrap(),
        ));
        let addr = FtColumn::Address(Some(Address::from([0x12; 32])));
        let asset_id = FtColumn::AssetId(Some(AssetId::from([0xA5; 32])));
        let bytes4 = FtColumn::Bytes4(Some(Bytes4::from([0xF0; 4])));
        let bytes8 = FtColumn::Bytes8(Some(Bytes8::from([0x9D; 8])));
        let bytes32 = FtColumn::Bytes32(Some(Bytes32::from([0xEE; 32])));
        let bytes64 = FtColumn::Bytes64(Some(Bytes64::from([0x12; 64])));
        let contractid = FtColumn::ContractId(Some(ContractId::from([0x78; 32])));
        let array = FtColumn::Array(Some(vec![FtColumn::I32(Some(1))]));
        let bytes = FtColumn::Bytes(Some(Bytes::from(vec![0u8, 1, 2, 3, 4, 5])));
        let identity = FtColumn::Identity(Some(Identity::Address([0x12; 32].into())));
        let int16 = FtColumn::I128(Some(i128::from_le_bytes([0x78; 16])));
        let int4 = FtColumn::I32(Some(i32::from_le_bytes([0x78; 4])));
        let int64 = FtColumn::I64(Some(i64::from_le_bytes([0x78; 8])));
        let int8 = FtColumn::I64(Some(i64::from_le_bytes([0x78; 8])));
        let json = FtColumn::Json(Some(Json(r#"{"hello":"world"}"#.to_string())));
        let r#bool = FtColumn::Boolean(Some(true));
        let r#enum = FtColumn::Enum(Some(String::from("hello")));
        let uint1 = FtColumn::U8(Some(u8::from_le_bytes([0x78; 1])));
        let uint16 = FtColumn::U128(Some(u128::from_le_bytes([0x78; 16])));
        let uint4 = FtColumn::U32(Some(u32::from_le_bytes([0x78; 4])));
        let uint8 = FtColumn::U64(Some(u64::from_le_bytes([0x78; 8])));
        let xstring = FtColumn::String(Some(String::from("hello world")));
        let uint2 = FtColumn::U16(Some(u16::from_le_bytes([0x78; 2])));
        let int2 = FtColumn::I16(Some(i16::from_le_bytes([0x78; 2])));

        insta::assert_yaml_snapshot!(addr.query_fragment());
        insta::assert_yaml_snapshot!(array.query_fragment());
        insta::assert_yaml_snapshot!(asset_id.query_fragment());
        insta::assert_yaml_snapshot!(bytes.query_fragment());
        insta::assert_yaml_snapshot!(bytes32.query_fragment());
        insta::assert_yaml_snapshot!(bytes4.query_fragment());
        insta::assert_yaml_snapshot!(bytes64.query_fragment());
        insta::assert_yaml_snapshot!(bytes8.query_fragment());
        insta::assert_yaml_snapshot!(contractid.query_fragment());
        insta::assert_yaml_snapshot!(identity.query_fragment());
        insta::assert_yaml_snapshot!(int16.query_fragment());
        insta::assert_yaml_snapshot!(int4.query_fragment());
        insta::assert_yaml_snapshot!(int64.query_fragment());
        insta::assert_yaml_snapshot!(int8.query_fragment());
        insta::assert_yaml_snapshot!(json.query_fragment());
        insta::assert_yaml_snapshot!(r#bool.query_fragment());
        insta::assert_yaml_snapshot!(r#enum.query_fragment());
        insta::assert_yaml_snapshot!(uid.query_fragment());
        insta::assert_yaml_snapshot!(uint1.query_fragment());
        insta::assert_yaml_snapshot!(uint16.query_fragment());
        insta::assert_yaml_snapshot!(uint4.query_fragment());
        insta::assert_yaml_snapshot!(uint8.query_fragment());
        insta::assert_yaml_snapshot!(xstring.query_fragment());
        insta::assert_yaml_snapshot!(uint2.query_fragment());
        insta::assert_yaml_snapshot!(int2.query_fragment());
    }

    #[test]
    fn test_fragments_none_types() {
        use super::*;

        let addr_none = FtColumn::Address(None);
        let array = FtColumn::Array(None);
        let asset_id_none = FtColumn::AssetId(None);
        let bytes = FtColumn::Bytes(None);
        let bytes32_none = FtColumn::Bytes32(None);
        let bytes4_none = FtColumn::Bytes4(None);
        let bytes64 = FtColumn::Bytes64(None);
        let bytes8_none = FtColumn::Bytes8(None);
        let contractid_none = FtColumn::ContractId(None);
        let identity_none = FtColumn::Identity(None);
        let int16_none = FtColumn::I64(None);
        let int4_none = FtColumn::I32(None);
        let int64_none = FtColumn::I64(None);
        let int8_none = FtColumn::I64(None);
        let json_none = FtColumn::Json(None);
        let r#bool = FtColumn::Boolean(None);
        let r#enum = FtColumn::Enum(None);
        let uint16_none = FtColumn::U64(None);
        let uint4_none = FtColumn::U32(None);
        let uint8_none = FtColumn::U64(None);
        let xstring_none = FtColumn::String(None);
        let uint2_none = FtColumn::U16(None);
        let int2_none = FtColumn::I16(None);

        insta::assert_yaml_snapshot!(addr_none.query_fragment());
        insta::assert_yaml_snapshot!(array.query_fragment());
        insta::assert_yaml_snapshot!(asset_id_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes.query_fragment());
        insta::assert_yaml_snapshot!(bytes32_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes4_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes64.query_fragment());
        insta::assert_yaml_snapshot!(bytes8_none.query_fragment());
        insta::assert_yaml_snapshot!(contractid_none.query_fragment());
        insta::assert_yaml_snapshot!(identity_none.query_fragment());
        insta::assert_yaml_snapshot!(int16_none.query_fragment());
        insta::assert_yaml_snapshot!(int4_none.query_fragment());
        insta::assert_yaml_snapshot!(int64_none.query_fragment());
        insta::assert_yaml_snapshot!(int8_none.query_fragment());
        insta::assert_yaml_snapshot!(json_none.query_fragment());
        insta::assert_yaml_snapshot!(r#bool.query_fragment());
        insta::assert_yaml_snapshot!(r#enum.query_fragment());
        insta::assert_yaml_snapshot!(uint16_none.query_fragment());
        insta::assert_yaml_snapshot!(uint4_none.query_fragment());
        insta::assert_yaml_snapshot!(uint8_none.query_fragment());
        insta::assert_yaml_snapshot!(xstring_none.query_fragment());
        insta::assert_yaml_snapshot!(uint2_none.query_fragment());
        insta::assert_yaml_snapshot!(int2_none.query_fragment());
    }

    #[test]
    #[should_panic(expected = "Schema fields of type `ID` cannot be nullable.")]
    fn test_panic_on_none_id_fragment() {
        use super::*;

        let uid_none = FtColumn::ID(None);
        insta::assert_yaml_snapshot!(uid_none.query_fragment());
    }
}
