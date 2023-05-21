//! # fuel-indexer-database-types
//!
//! `fuel-indexer-database-types` is a collection of data models used to create SQL tables
//!  from parsed GraphQL schema.

#![deny(unused_crate_dependencies)]

use crate::directives::IndexMethod;
use chrono::{serde::ts_microseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fmt::Write,
    string::ToString,
    time::{SystemTime, UNIX_EPOCH},
};
use strum::{AsRefStr, EnumString};

pub mod directives;

/// Root column used to identify to which graph registry a given column belongs.
#[derive(Debug)]
pub struct RootColumns {
    pub id: i64,
    pub root_id: i64,
    pub column_name: String,
    pub graphql_type: String,
}

/// New root column data model.
#[derive(Debug)]
pub struct NewRootColumns {
    pub root_id: i64,
    pub column_name: String,
    pub graphql_type: String,
}

/// Represents a graph root.
#[derive(Debug)]
pub struct GraphRoot {
    pub id: i64,
    pub version: String,
    pub schema_name: String,
    pub schema_identifier: String,
    pub schema: String,
}

/// Graph root data model.
#[derive(Debug)]
pub struct NewGraphRoot {
    pub version: String,
    pub schema_name: String,
    pub schema_identifier: String,
    pub schema: String,
}

/// A database column that does not result in SQL tables being generated.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualColumn {
    pub name: String,
    pub graphql_type: String,
}

/// Represents a database type within GraphQL schema.
#[derive(Debug)]
pub struct TypeId {
    pub id: i64,
    pub schema_version: String,
    pub schema_name: String,
    pub schema_identifier: String,
    pub graphql_name: String,
    pub table_name: String,
    pub virtual_columns: Vec<VirtualColumn>,
}

impl TypeId {
    /// Determine whether or not this type can be used to create SQL tables.
    pub fn is_non_indexable_type(&self) -> bool {
        !self.virtual_columns.is_empty()
    }
}

#[derive(Debug)]
pub struct IdLatest {
    pub schema_version: String,
}

/// Represents the number of versions for a given shcema.
#[derive(Debug)]
pub struct NumVersions {
    pub num: Option<i64>,
}

/// Represents a database column.
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

/// Similar to `NewColumn`, but is used to create the SQL
/// from which the actual database column is created
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
    /// Create a SQL fragment for the given column.
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

    /// Derive the respective PostgreSQL field type for a given `NewColumn`
    ///
    /// Here we're essentially matching `ColumnType`s to PostgreSQL field
    /// types. Note that we're using `numeric` field types for integer-like
    /// fields due to the ability to specify custom scale and precision. Some
    /// crates don't play well with unsigned integers (e.g., `sqlx`), so we
    /// just define these types as `numeric`, then convert them into their base
    /// types (e.g., u64) using `BigDecimal`.
    fn sql_type(&self) -> &str {
        match ColumnType::from(self.column_type.as_str()) {
            ColumnType::ID => "numeric(20, 0) primary key",
            ColumnType::Address => "varchar(64)",
            ColumnType::Bytes4 => "varchar(8)",
            ColumnType::Bytes8 => "varchar(16)",
            ColumnType::Bytes32 => "varchar(64)",
            ColumnType::AssetId => "varchar(64)",
            ColumnType::ContractId => "varchar(64)",
            ColumnType::Salt => "varchar(64)",
            ColumnType::Int4 => "integer",
            ColumnType::Int8 => "bigint",
            ColumnType::Int16 => "numeric(39, 0)",
            ColumnType::UInt4 | ColumnType::BlockHeight => "integer",
            ColumnType::UInt8 => "numeric(20, 0)",
            ColumnType::UInt16 => "numeric(39, 0)",
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
            ColumnType::Bytes64 => "varchar(128)",
            ColumnType::Signature => "varchar(128)",
            ColumnType::Nonce => "varchar(64)",
            ColumnType::HexString => "varchar(10485760)",
            ColumnType::Tai64Timestamp => "varchar(128)",
            ColumnType::TxId => "varchar(64)",
            ColumnType::Enum => "varchar(255)",
            ColumnType::Int1 => "integer",
            ColumnType::UInt1 => "integer",
        }
    }
}

/// Metadata about a given column.
#[derive(Debug)]
pub struct ColumnInfo {
    pub type_id: i64,
    pub table_name: String,
    pub column_position: i32,
    pub column_name: String,
    pub column_type: String,
}

/// Represents all types that can be persisted into the database.
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
    UInt16 = 21,
    Int16 = 22,
    Bytes64 = 23,
    Signature = 24,
    Nonce = 25,
    HexString = 26,
    Tai64Timestamp = 27,
    TxId = 28,
    BlockHeight = 29,
    Enum = 30,
    Int1 = 31,
    UInt1 = 32,
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
            ColumnType::UInt16 => 21,
            ColumnType::Int16 => 22,
            ColumnType::Bytes64 => 23,
            ColumnType::Signature => 24,
            ColumnType::Nonce => 25,
            ColumnType::HexString => 26,
            ColumnType::Tai64Timestamp => 27,
            ColumnType::TxId => 28,
            ColumnType::BlockHeight => 29,
            ColumnType::Enum => 30,
            ColumnType::Int1 => 31,
            ColumnType::UInt1 => 32,
        }
    }
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
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
            21 => ColumnType::Int16,
            22 => ColumnType::UInt16,
            23 => ColumnType::Bytes64,
            24 => ColumnType::Signature,
            25 => ColumnType::Nonce,
            26 => ColumnType::HexString,
            27 => ColumnType::Tai64Timestamp,
            28 => ColumnType::TxId,
            29 => ColumnType::BlockHeight,
            30 => ColumnType::Enum,
            31 => ColumnType::Int1,
            32 => ColumnType::UInt1,
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
            "UInt16" => ColumnType::UInt16,
            "Int16" => ColumnType::Int16,
            "Bytes64" => ColumnType::Bytes64,
            "Signature" => ColumnType::Signature,
            "Nonce" => ColumnType::Nonce,
            "HexString" => ColumnType::HexString,
            "Tai64Timestamp" => ColumnType::Tai64Timestamp,
            "TxId" => ColumnType::TxId,
            "BlockHeight" => ColumnType::BlockHeight,
            "Enum" => ColumnType::Enum,
            "Int1" => ColumnType::Int1,
            "UInt1" => ColumnType::UInt1,
            _ => panic!("Invalid column type: '{name}'"),
        }
    }
}

/// Represents an indexer asset (e.g., schema, manifest, WASM binary)
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerAsset {
    pub id: i64,
    pub index_id: i64,
    pub version: i32,
    pub digest: String,
    #[serde(skip_serializing)]
    pub bytes: Vec<u8>,
}

/// Represents all assets for a given indexer.
#[derive(Debug)]
pub struct IndexerAssetBundle {
    pub schema: IndexerAsset,
    pub manifest: IndexerAsset,
    pub wasm: IndexerAsset,
}

/// Represents the distinct types of assets that can be associated with an indexer.
#[derive(Debug, Eq, PartialEq, Hash, Clone, EnumString, AsRefStr)]
pub enum IndexerAssetType {
    #[strum(serialize = "wasm")]
    Wasm,
    #[strum(serialize = "manifest")]
    Manifest,
    #[strum(serialize = "schema")]
    Schema,
}

/// Represents a registered indexer.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisteredIndexer {
    pub id: i64,
    pub namespace: String,
    pub identifier: String,
    pub pubkey: Option<String>,
    #[serde(with = "ts_microseconds")]
    pub created_at: DateTime<Utc>,
}

impl RegisteredIndexer {
    /// Get the fully qualified identifier for this indexer.
    pub fn uid(&self) -> String {
        format!("{}.{}", self.namespace, self.identifier)
    }
}

/// Represents each type of database supported by the Fuel indexer.
#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub enum DbType {
    #[default]
    Postgres,
}

impl DbType {
    /// Get the fully qualified table name for a given table.
    pub fn table_name(&self, namespace: &str, table_name: &str) -> String {
        match self {
            DbType::Postgres => format!("{namespace}.{table_name}"),
        }
    }
}

/// Encapsulates all logic concerned with creating a given SQL abstraction.
pub trait CreateStatement {
    fn create_statement(&self) -> String;
}

/// Represents a SQL index on a given column.
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
    /// Return the index name for this column.
    pub fn name(&self) -> String {
        format!("{}_{}_idx", &self.table_name, &self.column_name)
    }
}

impl CreateStatement for ColumnIndex {
    /// Generate the create statement for this index.
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
        }

        frag
    }
}

/// Represents the SQL 'ON DELETE' action
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

/// Represents the SQL 'ON UPDATE' action
#[derive(Debug, Clone, Copy, Default, EnumString, AsRefStr)]
pub enum OnUpdate {
    #[default]
    #[strum(serialize = "NO ACTION")]
    NoAction,
}

/// Represents a SQL foreign key constraint.
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
    /// Create a new foreign key constraint.
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

    /// Get the fully qualified name for this foreign key.
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
    /// Generate the create statement for this foreign key.
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
        }
    }
}

/// Holder type of the 'id' column.
pub struct IdCol {}

impl IdCol {
    /// Get the ID column in lowercase.
    pub fn to_lowercase_string() -> String {
        "id".to_string()
    }

    /// Get the ID column in uppercase.
    pub fn to_uppercase_string() -> String {
        "ID".to_string()
    }
}

/// Represents a nonce for authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct Nonce {
    pub uid: String,
    pub expiry: i64,
}

impl Nonce {
    /// Determine whether the given nonce has expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        now >= self.expiry
    }
}
