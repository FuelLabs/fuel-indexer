extern crate alloc;
use alloc::vec::Vec;
pub use fuel_indexer_database_types as sql_types;

use crate::sql_types::ColumnType;
use core::convert::TryInto;
use graphql_parser::schema::{Definition, Document, TypeDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const LOG_LEVEL_ERROR: u32 = 0;
pub const LOG_LEVEL_WARN: u32 = 1;
pub const LOG_LEVEL_INFO: u32 = 2;
pub const LOG_LEVEL_DEBUG: u32 = 3;
pub const LOG_LEVEL_TRACE: u32 = 4;

use sha2::{Digest, Sha256};

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");

#[cfg(feature = "db-models")]
pub mod db;

pub mod types {

    use fuel_tx::Receipt;
    use serde::{Deserialize, Serialize};

    pub use fuels_core::types::Bits256;

    pub use fuel_types::{
        Address, AssetId, Bytes32, Bytes4, Bytes8, ContractId, Salt, Word,
    };

    pub const FUEL_TYPES_NAMESPACE: &str = "fuel";

    pub enum ReceiptType {
        Log,
        LogData,
        Transfer,
        TransferOut,
        ScriptResult,
    }

    impl From<String> for ReceiptType {
        fn from(s: String) -> Self {
            match s.as_str() {
                "Log" => ReceiptType::Log,
                "LogData" => ReceiptType::LogData,
                "Transfer" => ReceiptType::Transfer,
                "TransferOut" => ReceiptType::TransferOut,
                "ScriptResult" => ReceiptType::ScriptResult,
                _ => panic!("Unrecognized ReceiptType"),
            }
        }
    }

    pub trait Identity {
        fn path_ident_str() -> &'static str;
    }

    // NOTE: We could also create ABI JSON files with these native Fuel indexer-macro types <( '.' )>
    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct BlockData {
        pub height: u64,
        pub id: Bytes32,
        pub time: i64,
        pub producer: Address,
        pub transactions: Vec<Vec<Receipt>>,
    }

    impl Identity for BlockData {
        fn path_ident_str() -> &'static str {
            "BlockData"
        }
    }

    impl BlockData {
        pub fn macro_attribute_ident_str() -> &'static str {
            "block"
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct Transfer {
        pub contract_id: ContractId,
        pub to: ContractId,
        pub amount: u64,
        pub asset_id: AssetId,
        pub pc: u64,
        pub is: u64,
    }

    impl Identity for Transfer {
        fn path_ident_str() -> &'static str {
            "Transfer"
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct Log {
        pub contract_id: ContractId,
        pub ra: u64,
        pub rb: u64,
    }

    impl Identity for Log {
        fn path_ident_str() -> &'static str {
            "Log"
        }
    }

    // Keeping for now, but I don't believe we need this
    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct LogData {
        pub contract_id: ContractId,
        pub data: Vec<u8>,
        pub rb: u64,
        pub len: u64,
        pub ptr: u64,
    }

    impl Identity for LogData {
        fn path_ident_str() -> &'static str {
            "LogData"
        }
    }

    #[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
    pub struct Jsonb(pub String);

    pub type ID = u64;
    pub type Int4 = i32;
    pub type Int8 = i64;
    pub type UInt4 = u32;
    pub type UInt8 = u64;
    pub type Timestamp = u64;
}

// serde_scale for now, can look at other options if necessary.
pub fn serialize(obj: &impl Serialize) -> Vec<u8> {
    bincode::serialize(obj).expect("Serialize failed")
}

pub fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, String> {
    match bincode::deserialize(bytes) {
        Ok(obj) => Ok(obj),
        Err(e) => Err(format!("Bincode serde error {:?}", e)),
    }
}

pub fn type_id(namespace: &str, type_name: &str) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(
        &Sha256::digest(format!("{}:{}", namespace, type_name).as_bytes())[..8],
    );
    u64::from_le_bytes(bytes)
}

pub fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}

pub fn type_name(typ: &TypeDefinition<String>) -> String {
    match typ {
        TypeDefinition::Scalar(obj) => obj.name.clone(),
        TypeDefinition::Object(obj) => obj.name.clone(),
        TypeDefinition::Interface(obj) => obj.name.clone(),
        TypeDefinition::Union(obj) => obj.name.clone(),
        TypeDefinition::Enum(obj) => obj.name.clone(),
        TypeDefinition::InputObject(obj) => obj.name.clone(),
    }
}

pub fn get_schema_types(ast: &Document<String>) -> (HashSet<String>, HashSet<String>) {
    let types: HashSet<String> = ast
        .definitions
        .iter()
        .filter_map(|def| {
            if let Definition::TypeDefinition(typ) = def {
                Some(typ)
            } else {
                None
            }
        })
        .map(type_name)
        .collect();

    let directives = ast
        .definitions
        .iter()
        .filter_map(|def| {
            if let Definition::DirectiveDefinition(dir) = def {
                Some(dir.name.clone())
            } else {
                None
            }
        })
        .collect();

    (types, directives)
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Hash)]
pub enum FtColumn {
    ID(u64),
    Address(types::Address),
    AssetId(types::AssetId),
    Bytes4(types::Bytes4),
    Bytes8(types::Bytes8),
    Bytes32(types::Bytes32),
    ContractId(types::ContractId),
    Int4(i32),
    Int8(i64),
    UInt4(u32),
    UInt8(u64),
    Timestamp(u64),
    Salt(types::Salt),
    Jsonb(types::Jsonb),
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
                let address = types::Address::try_from(&bytes[..size])
                    .expect("Invalid slice length");
                FtColumn::Address(address)
            }
            ColumnType::AssetId => {
                let asset_id = types::AssetId::try_from(&bytes[..size])
                    .expect("Invalid slice length");
                FtColumn::AssetId(asset_id)
            }
            ColumnType::Bytes4 => {
                let bytes = types::Bytes4::try_from(&bytes[..size])
                    .expect("Invalid slice length");
                FtColumn::Bytes4(bytes)
            }
            ColumnType::Bytes8 => {
                let bytes = types::Bytes8::try_from(&bytes[..size])
                    .expect("Invalid slice length");
                FtColumn::Bytes8(bytes)
            }
            ColumnType::Bytes32 => {
                let bytes = types::Bytes32::try_from(&bytes[..size])
                    .expect("Invalid slice length");
                FtColumn::Bytes32(bytes)
            }
            ColumnType::ContractId => {
                let contract_id = types::ContractId::try_from(&bytes[..size])
                    .expect("Invalid slice length");
                FtColumn::ContractId(contract_id)
            }
            ColumnType::Salt => {
                let salt =
                    types::Salt::try_from(&bytes[..size]).expect("Invalid slice length");
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
                let uint8 = u64::from_le_bytes(
                    bytes[..size].try_into().expect("Invalid slice length"),
                );
                FtColumn::Timestamp(uint8)
            }
            ColumnType::Blob => {
                panic!("Blob not supported for FtColumn.");
            }
            ColumnType::ForeignKey => {
                panic!("ForeignKey not supported for FtColumn.");
            }
            ColumnType::Jsonb => FtColumn::Jsonb(types::Jsonb(
                String::from_utf8_lossy(&bytes[..size]).to_string(),
            )),
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
        let uint64 = FtColumn::Timestamp(u64::from_le_bytes([0x78; 8]));
        let salt = FtColumn::Salt(Salt::try_from([0x31; 32]).expect("Bad bytes"));

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
        insta::assert_yaml_snapshot!(uint64.query_fragment());
    }
}
