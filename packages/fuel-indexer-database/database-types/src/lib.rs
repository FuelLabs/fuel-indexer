#![deny(unused_crate_dependencies)]
pub mod directives;

use crate::directives::IndexMethod;
use async_graphql_parser::types::{FieldDefinition, TypeDefinition};
use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
use fuel_indexer_lib::graphql::ParsedGraphQLSchema;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fmt::Write,
    string::ToString,
    time::{SystemTime, UNIX_EPOCH},
};
use strum::{AsRefStr, EnumString};

/// SQL database types used by indexers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, AsRefStr)]
pub enum ColumnType {
    #[default]
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
    Virtual = 33,
    BlockId = 34,
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
            ColumnType::Virtual => 33,
            ColumnType::BlockId => 34,
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
            33 => ColumnType::Virtual,
            34 => ColumnType::BlockId,
            _ => panic!("Invalid ColumnType."),
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
            "Virtual" => ColumnType::Virtual,
            "BlockId" => ColumnType::BlockId,
            _ => panic!("Invalid ColumnType: '{name}'"),
        }
    }
}

/// Represents a root column in the graph.
#[derive(Debug, Default)]
pub struct RootColumns {
    /// Database ID of the column.
    pub id: i64,

    /// Database ID of the `GraphRoot` associated with this `RootColumns`.
    pub root_id: i64,

    /// Column name.
    pub column_name: String,

    /// GraphQL type of the column.
    pub graphql_type: String,
}

/// Whether a given column is virtual or regular. Virtual columns are not
/// persisted to the database.
#[derive(Debug, Default, EnumString, AsRefStr)]
pub enum ColumnPersistence {
    /// Virtual columns are not persisted to the database. They are represented
    /// by some arbitrarily sized type (e.g., JSON).
    Virtual,
    #[default]

    /// Regular columns are persisted to the database.
    Regular,
}

/// SQL statements that can be executed against a database.
pub trait SqlFragment {
    fn create(&self) -> String;
}

/// A named SQL statement.
pub trait SqlNamed {
    fn sql_name(&self) -> String;
}

/// Column on SQL database for a given `Table` the database.
#[derive(Debug, Default)]
pub struct FooColumn {
    /// Database ID of the column.
    pub id: i64,

    /// Database ID of the `TypeId` associated with this `FooColumn`.
    pub type_id: i64,

    /// Name of the column.
    pub name: String,

    /// GraphQL type of the column.
    pub graphql_type: String,

    /// SQL type of the column.
    pub coltype: ColumnType,

    /// Position of the column.
    ///
    /// Used when calling `fuel_indexer::database::Database::put_object`.
    pub position: i32,

    /// How this column is persisted to the database.
    pub persistence: ColumnPersistence,

    /// Whether this column is unique.
    pub unique: bool,

    /// Whether this column is nullable.
    pub nullable: bool,
}

impl FooColumn {
    #[allow(unused)]
    pub fn from_field_def(
        f: &FieldDefinition,
        namespace: &str,
        identifier: &str,
        version: &str,
        type_id: i64,
        position: i32,
    ) -> Self {
        let unique = f
            .directives
            .iter()
            .any(|d| d.node.name.to_string() == "unique");

        let table_name = f.name.to_string();
        Self {
            type_id,
            name: table_name.clone(),
            graphql_type: table_name,
            coltype: ColumnType::from(f.ty.to_string().as_str()),
            position,
            unique,
            nullable: f.ty.node.nullable,
            ..Self::default()
        }
    }

    /// Derive the respective PostgreSQL field type for a given `Columns`
    ///
    /// Here we're essentially matching `ColumnType`s to PostgreSQL field
    /// types. Note that we're using `numeric` field types for integer-like
    /// fields due to the ability to specify custom scale and precision. Some
    /// crates don't play well with unsigned integers (e.g., `sqlx`), so we
    /// just define these types as `numeric`, then convert them into their base
    /// types (e.g., u64) using `BigDecimal`.
    fn sql_type(&self) -> &str {
        match self.coltype {
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
            ColumnType::Virtual => "Json",
            ColumnType::BlockId => "varchar(64)",
        }
    }
}

impl SqlFragment for FooColumn {
    fn create(&self) -> String {
        let null_frag = if self.nullable { "" } else { "not null" };
        let unique_frag = if self.unique { "unique" } else { "" };
        format!(
            "{} {} {} {}",
            self.name,
            self.sql_type(),
            null_frag,
            unique_frag
        )
        .trim()
        .to_string()
    }
}

/// Represents the root of a graph in the database.
#[derive(Debug, Default)]
pub struct GraphRoot {
    /// Database ID of the root.
    pub id: i64,

    /// GraphQL schema version associated with this root.
    pub version: String,

    /// Indexer namespace associated with this root.
    pub schema_name: String,

    /// Indexer identifier associated with this root.
    pub schema_identifier: String,

    /// Raw GraphQL schema content.
    pub schema: String,
}

// // REFACTOR: remove
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct VirtualColumn {
//     pub name: String,
//     pub graphql_type: String,
// }

// // REFACTOR: remove
// #[derive(Debug)]
// pub struct TypeId {
//     pub id: i64,
//     pub schema_version: String,
//     pub schema_name: String,
//     pub schema_identifier: String,
//     pub graphql_name: String,
//     pub table_name: String,
//     pub virtual_columns: Vec<VirtualColumn>,
// }

/// Type ID used to identify `TypeDefintion`s in the GraphQL schema.
#[derive(Debug)]
pub struct FooTypeId {
    /// Database ID of the type.
    pub id: i64,

    /// Schema version associated with this type.
    pub version: String,

    /// Indexer namespace associated with this type.
    pub namespace: String,

    /// Indexer identifier associated with this type.
    pub identifier: String,

    /// GraphQL name of the type.
    pub graphql_name: String,

    /// Database table name of the type.
    pub table_name: String,

    /// How this type is persisted to the database.
    pub persistence: ColumnPersistence,
}

impl FooTypeId {
    pub fn from_typedef(typ: TypeDefinition, parsed: &ParsedGraphQLSchema) {
        unimplemented!()
    }
}

impl FooTypeId {
    pub fn from_field_def(
        f: &FieldDefinition,
        namespace: &str,
        identifier: &str,
        version: &str,
    ) -> Self {
        let persistence = f
            .directives
            .iter()
            .find(|d| d.node.name.to_string() == "virtual")
            .map(|_| ColumnPersistence::Virtual)
            .unwrap_or(ColumnPersistence::Regular);

        let table_name = f.name.to_string();
        Self {
            id: i64::MAX,
            version: version.to_string(),
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            graphql_name: f.name.to_string(),
            table_name,
            persistence,
        }
    }
}

// // REFACTOR: remove
// impl TypeId {
//     pub fn is_non_indexable_type(&self) -> bool {
//         !self.virtual_columns.is_empty()
//     }
// }

// // REFACTOR: remove
// #[derive(Debug)]
// pub struct Columns {
//     pub id: i64,
//     pub type_id: i64,
//     pub column_position: i32,
//     pub column_name: String,
//     pub column_type: String,
//     pub nullable: bool,
//     pub graphql_type: String,
//     pub unique: bool,
// }

// // REFACTOR: remove
// impl Columns {
//     pub fn create(&self) -> String {
//         let null_frag = if self.nullable { "" } else { "not null" };
//         let unique_frag = if self.unique { "unique" } else { "" };
//         format!(
//             "{} {} {} {}",
//             self.column_name,
//             self.sql_type(),
//             null_frag,
//             unique_frag
//         )
//         .trim()
//         .to_string()
//     }

//     /// Derive the respective PostgreSQL field type for a given `Columns`
//     ///
//     /// Here we're essentially matching `ColumnType`s to PostgreSQL field
//     /// types. Note that we're using `numeric` field types for integer-like
//     /// fields due to the ability to specify custom scale and precision. Some
//     /// crates don't play well with unsigned integers (e.g., `sqlx`), so we
//     /// just define these types as `numeric`, then convert them into their base
//     /// types (e.g., u64) using `BigDecimal`.
//     fn sql_type(&self) -> &str {
//         match ColumnType::from(self.column_type.as_str()) {
//             ColumnType::ID => "numeric(20, 0) primary key",
//             ColumnType::Address => "varchar(64)",
//             ColumnType::Bytes4 => "varchar(8)",
//             ColumnType::Bytes8 => "varchar(16)",
//             ColumnType::Bytes32 => "varchar(64)",
//             ColumnType::AssetId => "varchar(64)",
//             ColumnType::ContractId => "varchar(64)",
//             ColumnType::Salt => "varchar(64)",
//             ColumnType::Int4 => "integer",
//             ColumnType::Int8 => "bigint",
//             ColumnType::Int16 => "numeric(39, 0)",
//             ColumnType::UInt4 | ColumnType::BlockHeight => "integer",
//             ColumnType::UInt8 => "numeric(20, 0)",
//             ColumnType::UInt16 => "numeric(39, 0)",
//             ColumnType::Timestamp => "timestamp",
//             ColumnType::Object => "bytea",
//             ColumnType::Blob => "varchar(10485760)",
//             ColumnType::ForeignKey => {
//                 panic!("ForeignKey ColumnType is a reference type only.")
//             }
//             ColumnType::Json => "Json",
//             ColumnType::MessageId => "varchar(64)",
//             ColumnType::Charfield => "varchar(255)",
//             ColumnType::Identity => "varchar(66)",
//             ColumnType::Boolean => "boolean",
//             ColumnType::Bytes64 => "varchar(128)",
//             ColumnType::Signature => "varchar(128)",
//             ColumnType::Nonce => "varchar(64)",
//             ColumnType::HexString => "varchar(10485760)",
//             ColumnType::Tai64Timestamp => "varchar(128)",
//             ColumnType::TxId => "varchar(64)",
//             ColumnType::Enum => "varchar(255)",
//             ColumnType::Int1 => "integer",
//             ColumnType::UInt1 => "integer",
//             ColumnType::Virtual => "Json",
//             ColumnType::BlockId => "varchar(64)",
//         }
//     }
// }

/// I don't actually know wtf this is for
#[derive(Debug)]
pub struct ColumnInfo {
    pub type_id: i64,
    pub table_name: String,
    pub column_position: i32,
    pub column_name: String,
    pub column_type: String,
}

/// Represents an asset that is used to create and identify indexers.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerAsset {
    /// Database ID of the asset.
    pub id: i64,

    /// Database ID of the indexer.
    pub index_id: i64,

    /// Version associated with this indexer asset.
    pub version: i32,

    /// Digest of the asset's bytes.
    pub digest: String,

    /// Bytes of the asset.
    #[serde(skip_serializing)]
    pub bytes: Vec<u8>,
}

/// Represents a bundle of assets that are used to create and identify indexers.
#[derive(Debug)]
pub struct IndexerAssetBundle {
    /// Indexer schema asset.
    pub schema: IndexerAsset,

    /// Indexer manifest asset.
    pub manifest: IndexerAsset,

    /// Indexer WASM asset.
    pub wasm: IndexerAsset,
}

/// All assets that can be used on indexers.
#[derive(Debug, Eq, PartialEq, Hash, Clone, EnumString, AsRefStr)]
pub enum IndexerAssetType {
    #[strum(serialize = "wasm")]
    Wasm,
    #[strum(serialize = "manifest")]
    Manifest,
    #[strum(serialize = "schema")]
    Schema,
}

/// An indexer that has been persisted to the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisteredIndexer {
    /// Database ID of the indexer.
    pub id: i64,

    /// Namespace of the indexer.
    pub namespace: String,

    /// Identifier of the indexer.
    pub identifier: String,

    /// Public key associated with this indexer.
    ///
    /// Only used if authentication is enabled.
    pub pubkey: Option<String>,

    /// Time at which indexer was created.
    #[serde(with = "ts_microseconds")]
    pub created_at: DateTime<Utc>,
}

impl RegisteredIndexer {
    pub fn uid(&self) -> String {
        format!("{}.{}", self.namespace, self.identifier)
    }
}

/// SQL database types used by indexers.
#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub enum DbType {
    #[default]
    Postgres,
}

impl DbType {
    /// Return the fully qualified table name for a given database type, namespace, and table name.
    pub fn table_name(&self, namespace: &str, table_name: &str) -> String {
        match self {
            DbType::Postgres => format!("{namespace}.{table_name}"),
        }
    }
}

/// SQL database index for a given column.
#[derive(Debug)]
pub struct SqlIndex {
    /// The type of database.
    pub db_type: DbType,

    /// Name of table index is applied to.
    pub table_name: String,

    /// Namespace of the indexer.
    pub namespace: String,

    /// SQL index method.
    pub method: IndexMethod,

    /// Whether this index is unique.
    pub unique: bool,

    /// Name of column index is applied to.
    pub column_name: String,
}

impl SqlNamed for SqlIndex {
    fn sql_name(&self) -> String {
        format!("{}_{}_idx", &self.table_name, &self.column_name)
    }
}

impl SqlFragment for SqlIndex {
    fn create(&self) -> String {
        let mut frag = "CREATE ".to_string();
        if self.unique {
            frag += "UNIQUE ";
        }

        match self.db_type {
            DbType::Postgres => {
                let _ = write!(
                    frag,
                    "INDEX {} ON {}.{} USING {} ({});",
                    "".to_string(),
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

/// On delete action for a FK constraint.
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

/// On update action for a FK constraint.
#[derive(Debug, Clone, Copy, Default, EnumString, AsRefStr)]
pub enum OnUpdate {
    #[default]
    #[strum(serialize = "NO ACTION")]
    NoAction,
}

/// SQL database constraint for a given column.
#[derive(Debug)]
pub enum Constraint {
    Index(SqlIndex),
    Fk(ForeignKey),
}

impl SqlFragment for Constraint {
    fn create(&self) -> String {
        match self {
            Constraint::Index(idx) => idx.create(),
            Constraint::Fk(fk) => fk.create(),
        }
    }
}

/// SQL database foreign key for a given column.
#[derive(Debug, Clone, Default)]
pub struct ForeignKey {
    /// The type of database.
    pub db_type: DbType,

    /// The namespace of the indexer.
    pub namespace: String,

    /// Name of table table FK is applied to.
    pub table_name: String,

    /// Name of column FK is applied to.
    pub column_name: String,

    /// Name of table FK references.
    pub ref_tablename: String,

    /// Name of column FK references.
    pub ref_colname: String,

    /// Type of column FK references.
    pub ref_coltype: String,

    /// Action to take on delete.
    pub on_delete: OnDelete,

    /// Action to take on update.
    pub on_update: OnUpdate,
}

impl ForeignKey {
    /// Create a new `ForeignKey`.
    pub fn new(
        db_type: DbType,
        namespace: String,
        table_name: String,
        column_name: String,
        ref_tablename: String,
        ref_column_name: String,
        ref_coltype: String,
    ) -> Self {
        Self {
            db_type,
            namespace,
            table_name,
            column_name,
            ref_colname: ref_column_name,
            ref_tablename,
            ref_coltype,
            ..Default::default()
        }
    }
}

impl SqlNamed for ForeignKey {
    fn sql_name(&self) -> String {
        format!(
            "fk_{}_{}__{}_{}",
            self.table_name, self.column_name, self.ref_tablename, self.ref_colname
        )
    }
}

impl SqlFragment for ForeignKey {
    fn create(&self) -> String {
        match self.db_type {
            DbType::Postgres => {
                format!(
                    "ALTER TABLE {}.{} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}({}) ON DELETE {} ON UPDATE {} INITIALLY DEFERRED;",
                    self.namespace,
                    self.table_name,
                    // Tmp
                    "".to_string(),
                    self.column_name,
                    self.namespace,
                    self.ref_tablename,
                    self.ref_colname,
                    self.on_delete.as_ref(),
                    self.on_update.as_ref()
                )
            }
        }
    }
}

/// Represents an ID field in a SQL column or GraphQL schema.s
pub struct IdCol;
impl IdCol {
    pub fn to_lowercase_string() -> String {
        "id".to_string()
    }

    pub fn to_lowercase_str() -> &'static str {
        "id"
    }

    pub fn to_uppercase_string() -> String {
        "ID".to_string()
    }

    pub fn to_uppercase_str() -> &'static str {
        "ID"
    }
}

/// Nonce used for indexer authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct Nonce {
    /// Unique string used as a nonce payload.
    pub uid: String,

    /// Expiry time of the nonce.
    pub expiry: i64,
}

impl Nonce {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        now >= self.expiry
    }
}

/// SQL database table for a given `GraphRoot` in the database.
pub struct Table {
    /// The name of the table.
    name: String,

    /// The namespace of the indexer.
    namespace: String,

    /// The identifier of the indexer.
    identifier: String,

    /// SQL columns associated with this table.
    columns: Vec<FooColumn>,

    /// SQL conswtraints associated with this table.
    constraints: Vec<Constraint>,

    /// How this table is persisted to the database.
    persistence: ColumnPersistence,
}

impl Table {
    pub fn constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }

    pub fn from_typdef(_typ: TypeDefinition, _parsed: &ParsedGraphQLSchema) -> Self {
        unimplemented!()
    }
}

impl SqlFragment for Table {
    fn create(&self) -> String {
        unimplemented!()
    }
}
