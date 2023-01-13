use crate::directives::IndexMethod;
use serde::{Deserialize, Serialize};
use std::{fmt, fmt::Write};
use strum::{AsRefStr, EnumString};

pub mod directives;

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
    pub unique: bool,
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
        let null_frag = if self.nullable { "" } else { "not null" };
        let unique_frag = if self.unique { "unique" } else { "" };
        format!(
            "{} {} {} {}",
            self.column_name,
            self.sql_type(),
            null_frag,
            unique_frag
        )
        .trim()
        .to_string()
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
            ColumnType::Object => "bytea",
            ColumnType::Blob => "varchar(10485760)",
            ColumnType::ForeignKey => {
                panic!("ForeignKey ColumnType is a reference type only.")
            }
            ColumnType::Json => "Json",
            ColumnType::MessageId => "varchar(64)",
            ColumnType::Charfield => "varchar(255)",
            ColumnType::Identity => "varchar(66)",
            ColumnType::Boolean => "boolean",
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
    Json = 15,
    MessageId = 16,
    Charfield = 17,
    Identity = 18,
    Boolean = 19,
    Object = 20,
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
            ColumnType::Json => 15,
            ColumnType::MessageId => 16,
            ColumnType::Charfield => 17,
            ColumnType::Identity => 18,
            ColumnType::Boolean => 19,
            ColumnType::Object => 20,
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
            15 => ColumnType::Json,
            16 => ColumnType::MessageId,
            17 => ColumnType::Charfield,
            18 => ColumnType::Identity,
            19 => ColumnType::Boolean,
            20 => ColumnType::Object,
            _ => panic!("Invalid column type."),
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
            "Json" => ColumnType::Json,
            "MessageId" => ColumnType::MessageId,
            "Charfield" => ColumnType::Charfield,
            "Identity" => ColumnType::Identity,
            "Boolean" => ColumnType::Boolean,
            "Object" => ColumnType::Object,
            _ => panic!("Invalid column type: '{}'", name),
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

#[derive(Debug, Eq, PartialEq, Hash, Clone, EnumString, AsRefStr)]
pub enum IndexAssetType {
    #[strum(serialize = "wasm")]
    Wasm,
    #[strum(serialize = "manifest")]
    Manifest,
    #[strum(serialize = "schema")]
    Schema,
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

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub enum DbType {
    #[default]
    Postgres,
    Sqlite,
}

impl DbType {
    pub fn table_name(&self, namespace: &str, table_name: &str) -> String {
        match self {
            DbType::Postgres => format!("{}.{}", namespace, table_name),
            DbType::Sqlite => table_name.to_string(),
        }
    }
}

pub trait CreateStatement {
    fn create_statement(&self) -> String;
}

#[derive(Debug)]
pub struct ColumnIndex {
    pub db_type: DbType,
    pub table_name: String,
    pub namespace: String,
    pub method: IndexMethod,
    pub unique: bool,
    pub column_name: String,
}

impl ColumnIndex {
    pub fn name(&self) -> String {
        format!("{}_{}_idx", &self.table_name, &self.column_name)
    }
}

impl CreateStatement for ColumnIndex {
    fn create_statement(&self) -> String {
        let mut frag = "CREATE ".to_string();
        if self.unique {
            frag += "UNIQUE ";
        }

        match self.db_type {
            DbType::Postgres => {
                let _ = write!(
                    frag,
                    "INDEX {} ON {}.{} USING {} ({});",
                    self.name(),
                    self.namespace,
                    self.table_name,
                    self.method.as_ref(),
                    self.column_name
                );
            }
            DbType::Sqlite => {
                let _ = write!(
                    frag,
                    "INDEX IF NOT EXISTS {} ON {}({});",
                    self.name(),
                    self.table_name,
                    self.column_name
                );
            }
        }

        frag
    }
}

#[derive(Debug, Clone, Copy, Default, EnumString, AsRefStr)]
pub enum OnDelete {
    #[default]
    #[strum(serialize = "NO ACTION")]
    NoAction,
    #[strum(serialize = "CASCADE")]
    Cascade,
    #[strum(serialize = "SET NULL")]
    SetNull,
}

#[derive(Debug, Clone, Copy, Default, EnumString, AsRefStr)]
pub enum OnUpdate {
    #[default]
    #[strum(serialize = "NO ACTION")]
    NoAction,
}

#[derive(Debug, Clone, Default)]
pub struct ForeignKey {
    pub db_type: DbType,
    pub namespace: String,
    pub table_name: String,
    pub column_name: String,
    pub reference_table_name: String,
    pub reference_column_name: String,
    pub reference_column_type: String,
    pub on_delete: OnDelete,
    pub on_update: OnUpdate,
}

impl ForeignKey {
    pub fn new(
        db_type: DbType,
        namespace: String,
        table_name: String,
        column_name: String,
        reference_table_name: String,
        ref_column_name: String,
        reference_column_type: String,
    ) -> Self {
        Self {
            db_type,
            namespace,
            table_name,
            column_name,
            reference_column_name: ref_column_name,
            reference_table_name,
            reference_column_type,
            ..Default::default()
        }
    }

    pub fn name(&self) -> String {
        format!(
            "fk_{}_{}__{}_{}",
            self.table_name,
            self.column_name,
            self.reference_table_name,
            self.reference_column_name
        )
    }
}

impl CreateStatement for ForeignKey {
    fn create_statement(&self) -> String {
        match self.db_type {
            DbType::Postgres => {
                format!(
                    "ALTER TABLE {}.{} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}({}) ON DELETE {} ON UPDATE {} INITIALLY DEFERRED;",
                    self.namespace,
                    self.table_name,
                    self.name(),
                    self.column_name,
                    self.namespace,
                    self.reference_table_name,
                    self.reference_column_name,
                    self.on_delete.as_ref(),
                    self.on_update.as_ref()
                )
            }
            DbType::Sqlite => {
                fn schema_type_to_sqlite_type(t: &str) -> String {
                    match t {
                        "ID" => "BIGINT".to_string(),
                        "UInt8" | "Int8" | "Int4" | "UInt4" => "INTEGER".to_string(),
                        _ => "TEXT".to_string(),
                    }
                }

                format!(
                    "ALTER TABLE {} DROP COLUMN {}; ALTER TABLE {} ADD COLUMN {} {} REFERENCES {}({});",
                    self.table_name,
                    self.column_name,
                    self.table_name,
                    self.column_name,
                    schema_type_to_sqlite_type(&self.reference_column_type),
                    self.reference_table_name,
                    self.reference_column_name,
                )
            }
        }
    }
}

//
pub struct IdCol {}
impl IdCol {
    pub fn to_lowercase_string() -> String {
        "id".to_string()
    }

    pub fn to_uppercase_string() -> String {
        "ID".to_string()
    }
}
