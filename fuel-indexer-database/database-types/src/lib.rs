use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub struct RootColumns {
    pub id: i64,
    pub root_id: i64,
    pub column_name: String,
    pub graphql_type: String,
}

#[derive(Debug)]
pub struct NewRootColumns {
    pub root_id: i64,
    pub column_name: String,
    pub graphql_type: String,
}

#[derive(Debug)]
pub struct GraphRoot {
    pub id: i64,
    pub version: String,
    pub schema_name: String,
    pub query: String,
    pub schema: String,
}

#[derive(Debug)]
pub struct NewGraphRoot {
    pub version: String,
    pub schema_name: String,
    pub query: String,
    pub schema: String,
}

#[derive(Debug)]
pub struct TypeId {
    pub id: i64,
    pub schema_version: String,
    pub schema_name: String,
    pub graphql_name: String,
    pub table_name: String,
}

#[derive(Debug)]
pub struct IdLatest {
    pub schema_version: String,
}

#[derive(Debug)]
pub struct NumVersions {
    pub num: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct NewColumn {
    pub type_id: i64,
    pub column_position: i32,
    pub column_name: String,
    pub column_type: String,
    pub nullable: bool,
    pub graphql_type: String,
}

#[derive(Debug)]
pub struct Columns {
    pub id: i64,
    pub type_id: i64,
    pub column_position: i32,
    pub column_name: String,
    pub column_type: String,
    pub nullable: bool,
    pub graphql_type: String,
}

impl NewColumn {
    pub fn sql_fragment(&self) -> String {
        if self.nullable {
            format!("{} {}", self.column_name, self.sql_type())
        } else {
            format!("{} {} not null", self.column_name, self.sql_type())
        }
    }

    fn sql_type(&self) -> &str {
        match ColumnType::from(self.column_type.as_str()) {
            ColumnType::ID => "bigint primary key",
            ColumnType::Address => "varchar(64)",
            ColumnType::Bytes4 => "varchar(8)",
            ColumnType::Bytes8 => "varchar(16)",
            ColumnType::Bytes32 => "varchar(64)",
            ColumnType::AssetId => "varchar(64)",
            ColumnType::ContractId => "varchar(64)",
            ColumnType::Salt => "varchar(64)",
            ColumnType::Int4 => "integer",
            ColumnType::Int8 => "bigint",
            ColumnType::UInt4 => "integer",
            ColumnType::UInt8 => "bigint",
            ColumnType::Timestamp => "timestamp",
            ColumnType::Blob => "bytea",
            ColumnType::ForeignKey => panic!("ForeignKey ColumnType is a reference type only."),
            ColumnType::Jsonb => "jsonb",
        }
    }
}

#[derive(Debug)]
pub struct ColumnInfo {
    pub type_id: i64,
    pub table_name: String,
    pub column_position: i32,
    pub column_name: String,
    pub column_type: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ColumnType {
    ID = 0,
    Address = 1,
    AssetId = 2,
    Bytes4 = 3,
    Bytes8 = 4,
    Bytes32 = 5,
    ContractId = 6,
    Salt = 7,
    Int4 = 8,
    Int8 = 9,
    UInt4 = 10,
    UInt8 = 11,
    Timestamp = 12,
    Blob = 13,
    ForeignKey = 14,
    Jsonb = 15,
}

impl From<ColumnType> for i32 {
    fn from(typ: ColumnType) -> i32 {
        match typ {
            ColumnType::ID => 0,
            ColumnType::Address => 1,
            ColumnType::AssetId => 2,
            ColumnType::Bytes4 => 3,
            ColumnType::Bytes8 => 4,
            ColumnType::Bytes32 => 5,
            ColumnType::ContractId => 6,
            ColumnType::Salt => 7,
            ColumnType::Int4 => 8,
            ColumnType::Int8 => 9,
            ColumnType::UInt4 => 10,
            ColumnType::UInt8 => 11,
            ColumnType::Timestamp => 12,
            ColumnType::Blob => 13,
            ColumnType::ForeignKey => 14,
            ColumnType::Jsonb => 15,
        }
    }
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<i32> for ColumnType {
    fn from(num: i32) -> ColumnType {
        match num {
            0 => ColumnType::ID,
            1 => ColumnType::Address,
            2 => ColumnType::AssetId,
            3 => ColumnType::Bytes4,
            4 => ColumnType::Bytes8,
            5 => ColumnType::Bytes32,
            6 => ColumnType::ContractId,
            7 => ColumnType::Salt,
            8 => ColumnType::Int4,
            9 => ColumnType::Int8,
            10 => ColumnType::UInt4,
            11 => ColumnType::UInt8,
            12 => ColumnType::Timestamp,
            13 => ColumnType::Blob,
            14 => ColumnType::ForeignKey,
            15 => ColumnType::Jsonb,
            _ => panic!("Invalid column type!"),
        }
    }
}

impl From<&str> for ColumnType {
    fn from(name: &str) -> ColumnType {
        match name {
            "ID" => ColumnType::ID,
            "Address" => ColumnType::Address,
            "AssetId" => ColumnType::AssetId,
            "Bytes4" => ColumnType::Bytes4,
            "Bytes8" => ColumnType::Bytes8,
            "Bytes32" => ColumnType::Bytes32,
            "ContractId" => ColumnType::ContractId,
            "Salt" => ColumnType::Salt,
            "Int4" => ColumnType::Int4,
            "Int8" => ColumnType::Int8,
            "UInt4" => ColumnType::UInt4,
            "UInt8" => ColumnType::UInt8,
            "Timestamp" => ColumnType::Timestamp,
            "Blob" => ColumnType::Blob,
            "ForeignKey" => ColumnType::ForeignKey,
            "Jsonb" => ColumnType::Jsonb,
            _ => panic!("Invalid column type! {}", name),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexAsset {
    pub id: i64,
    pub index_id: i64,
    pub version: i32,
    pub digest: String,
    #[serde(skip_serializing)]
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct IndexAssetBundle {
    pub schema: IndexAsset,
    pub manifest: IndexAsset,
    pub wasm: IndexAsset,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum IndexAssetType {
    Wasm,
    Manifest,
    Schema,
}

impl IndexAssetType {
    pub fn as_str(&self) -> &str {
        match self {
            IndexAssetType::Wasm => "wasm",
            IndexAssetType::Manifest => "manifest",
            IndexAssetType::Schema => "schema",
        }
    }
}

impl ToString for IndexAssetType {
    fn to_string(&self) -> String {
        match self {
            IndexAssetType::Wasm => "wasm".to_string(),
            IndexAssetType::Manifest => "manifest".to_string(),
            IndexAssetType::Schema => "schema".to_string(),
        }
    }
}

impl From<&str> for IndexAssetType {
    fn from(a: &str) -> Self {
        match a {
            "wasm" => Self::Wasm,
            "manifest" => Self::Manifest,
            "schema" => Self::Schema,
            _ => panic!("Unrecognized IndexAssetType."),
        }
    }
}

impl From<String> for IndexAssetType {
    fn from(a: String) -> Self {
        match a.as_str() {
            "wasm" => Self::Wasm,
            "manifest" => Self::Manifest,
            "schema" => Self::Schema,
            _ => panic!("Unrecognized IndexAssetType."),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisteredIndex {
    pub id: i64,
    pub namespace: String,
    pub identifier: String,
}

impl RegisteredIndex {
    pub fn uid(&self) -> String {
        format!("{}.{}", self.namespace, self.identifier)
    }
}
