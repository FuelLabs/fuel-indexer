#![deny(unused_crate_dependencies)]
use async_graphql_parser::{
    types::{FieldDefinition, ObjectType, TypeDefinition, TypeKind},
    Pos, Positioned,
};
use async_graphql_value::Name;
use chrono::{
    serde::ts_microseconds,
    {DateTime, Utc},
};
use fuel_indexer_lib::{
    graphql::{
        extract_foreign_key_info, field_id, is_list_type,
        types::{IdCol, ObjectCol},
        JoinTableItem, ParsedGraphQLSchema,
    },
    type_id,
};
use linked_hash_set::LinkedHashSet;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt,
    fmt::Write,
    string::ToString,
    time::{SystemTime, UNIX_EPOCH},
};
use strum::{AsRefStr, EnumString};

// SQL index method.
#[derive(Debug, EnumString, AsRefStr, Default)]
pub enum IndexMethod {
    #[default]
    #[strum(serialize = "btree")]
    BTree,

    #[strum(serialize = "hash")]
    Hash,
}

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
    Array = 35,
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
            ColumnType::Array => 35,
        }
    }
}

impl From<ColumnType> for i64 {
    fn from(typ: ColumnType) -> i64 {
        let typ = i32::from(typ);
        typ as i64
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
            35 => ColumnType::Array,
            _ => unimplemented!("Invalid ColumnType: {num}."),
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
            "Array" => ColumnType::Array,
            _ => unimplemented!("Invalid ColumnType: '{name}'."),
        }
    }
}

/// Represents a root column in the graph.
#[derive(Debug, Default)]
pub struct RootColumn {
    /// Database ID of the column.
    pub id: i64,

    /// Database ID of the `GraphRoot` associated with this `RootColumn`.
    pub root_id: i64,

    /// Column name.
    pub column_name: String,

    /// GraphQL type of the column.
    pub graphql_type: String,
}

/// How the column is persisted to the DB.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Default, AsRefStr, strum::Display, EnumString,
)]
pub enum Persistence {
    /// Virtual columns are not persisted to the database. They are represented
    /// by some arbitrarily sized type (e.g., JSON).
    #[default]
    Virtual,

    /// Scalar columns are persisted to the database.
    Scalar,
}

/// SQL statements that can be executed against a database.
pub trait SqlFragment {
    /// Return the SQL create statement for a given SQL item.
    fn create(&self) -> String;
}

/// A named SQL statement.
pub trait SqlNamed {
    /// Return the SQL name of the SQL item.
    fn sql_name(&self) -> String;
}

/// Column on SQL database for a given `Table` the database.
#[derive(Debug, Default, Clone)]
pub struct Column {
    /// Database ID of the column.
    pub id: i64,

    /// Database ID of the `TypeId` associated with this `Column`.
    pub type_id: i64,

    /// Name of the column.
    pub name: String,

    /// GraphQL type of the column.
    pub graphql_type: String,

    /// SQL type of the column.
    pub coltype: ColumnType,

    /// Position of the column.
    ///
    /// Used when determing the order of columns when saving objects to DB
    /// and retrieving objects from DB.
    pub position: i32,

    /// How this column is persisted to the database.
    pub persistence: Persistence,

    /// Whether this column is unique.
    pub unique: bool,

    /// Whether this column is nullable.
    pub nullable: bool,

    /// SQL type of the array's contents
    ///
    /// Only if this is a `ColumnType::Array`
    pub array_coltype: Option<ColumnType>,
}

impl SqlNamed for Column {
    /// Return the SQL name of the column.
    fn sql_name(&self) -> String {
        self.name.to_string()
    }
}

impl Column {
    /// Create a new `Column` from a given `FieldDefinition`.
    pub fn from_field_def(
        f: &FieldDefinition,
        parsed: &ParsedGraphQLSchema,
        type_id: i64,
        position: i32,
        persistence: Persistence,
    ) -> Self {
        let mut field_type = f.ty.to_string().replace(['[', ']', '!'], "");
        if parsed.is_possible_foreign_key(&field_type) {
            field_type = f
            .directives
            .iter()
            .find(|d| d.node.name.to_string() == "join")
            .map(|d| {
                let ref_field_name =
                    d.clone().node.arguments.pop().unwrap().1.to_string();
                let fk_fid = field_id(&field_type, &ref_field_name);
                let fk_field_typ = parsed
                    .field_type_mappings()
                    .get(&fk_fid)
                    .expect("Failed to find field in ParsedGraphQLSchema field type mappings.")
                    .to_string();
                fk_field_typ
            })
            // Special case of parsing FKs here where we change the derived
            // field type. We can't use the `ID` type as normal because we
            // can't have multiple primary keys on the same table.
            .unwrap_or("UInt8".to_string());
        }

        match is_list_type(f) {
            true => Self {
                type_id,
                name: f.name.to_string(),
                graphql_type: format!("[{field_type}]"),
                coltype: ColumnType::Array,
                position,
                array_coltype: Some(ColumnType::from(field_type.as_str())),
                nullable: f.ty.node.nullable,
                persistence,
                ..Self::default()
            },
            false => {
                if parsed.is_possible_foreign_key(&field_type) {
                    // Determine implicit vs explicit FK type
                    field_type = f
                        .directives
                        .iter()
                        .find(|d| d.node.name.to_string() == "join")
                        .map(|d| {
                            let ref_field_name =
                                d.clone().node.arguments.pop().unwrap().1.to_string();
                            let fk_fid = field_id(&field_type, &ref_field_name);
                            let fk_field_typ = parsed
                                .field_type_mappings()
                                .get(&fk_fid)
                                .expect("Failed to find field in ParsedGraphQLSchema field type mappings.")
                                .to_string();
                            fk_field_typ
                        })
                        // Special case of parsing FKs here where we change the derived
                        // field type. We can't use the `ID` type as normal because we
                        // can't have multiple primary keys on the same table.
                        .unwrap_or("UInt8".to_string());
                } else if parsed.is_virtual_typedef(&field_type) {
                    field_type = "Virtual".to_string();
                } else if parsed.is_enum_typedef(&field_type) {
                    field_type = "Charfield".to_string();
                }

                if is_list_type(f) {
                    field_type = "Array".to_string();
                }

                let unique = f
                    .directives
                    .iter()
                    .any(|d| d.node.name.to_string() == "unique");

                let coltype = field_type.as_str();

                Self {
                    type_id,
                    name: f.name.to_string(),
                    graphql_type: coltype.to_owned(),
                    coltype: ColumnType::from(coltype),
                    position,
                    unique,
                    nullable: f.ty.node.nullable,
                    persistence,
                    ..Self::default()
                }
            }
        }
    }

    /// Derive the respective PostgreSQL field type for a given `Columns`
    fn sql_type(&self) -> String {
        // Here we're essentially matching `ColumnType`s to PostgreSQL field
        // types. Note that we're using `numeric` field types for integer-like
        // fields due to the ability to specify custom scale and precision. Some
        // crates don't play well with unsigned integers (e.g., `sqlx`), so we
        // just define these types as `numeric`, then convert them into their base
        // types (e.g., u64) using `BigDecimal`.
        match self.coltype {
            ColumnType::ID => "numeric(20, 0) primary key".to_string(),
            ColumnType::Address => "varchar(64)".to_string(),
            ColumnType::Bytes4 => "varchar(8)".to_string(),
            ColumnType::Bytes8 => "varchar(16)".to_string(),
            ColumnType::Bytes32 => "varchar(64)".to_string(),
            ColumnType::AssetId => "varchar(64)".to_string(),
            ColumnType::ContractId => "varchar(64)".to_string(),
            ColumnType::Salt => "varchar(64)".to_string(),
            ColumnType::Int4 => "integer".to_string(),
            ColumnType::Int8 => "bigint".to_string(),
            ColumnType::Int16 => "numeric(39, 0)".to_string(),
            ColumnType::UInt4 | ColumnType::BlockHeight => "integer".to_string(),
            ColumnType::UInt8 => "numeric(20, 0)".to_string(),
            ColumnType::UInt16 => "numeric(39, 0)".to_string(),
            ColumnType::Timestamp => "timestamp".to_string(),
            ColumnType::Object => "bytea".to_string(),
            ColumnType::Blob => "varchar(10485760)".to_string(),
            ColumnType::ForeignKey => "numeric(20, 0)".to_string(),
            ColumnType::Json => "json".to_string(),
            ColumnType::MessageId => "varchar(64)".to_string(),
            ColumnType::Charfield => "varchar(255)".to_string(),
            ColumnType::Identity => "varchar(66)".to_string(),
            ColumnType::Boolean => "boolean".to_string(),
            ColumnType::Bytes64 => "varchar(128)".to_string(),
            ColumnType::Signature => "varchar(128)".to_string(),
            ColumnType::Nonce => "varchar(64)".to_string(),
            ColumnType::HexString => "varchar(10485760)".to_string(),
            ColumnType::Tai64Timestamp => "varchar(128)".to_string(),
            ColumnType::TxId => "varchar(64)".to_string(),
            ColumnType::Enum => "varchar(255)".to_string(),
            ColumnType::Int1 => "integer".to_string(),
            ColumnType::UInt1 => "integer".to_string(),
            ColumnType::Virtual => "json".to_string(),
            ColumnType::BlockId => "varchar(64)".to_string(),
            ColumnType::Array => {
                let t = match self.array_coltype.expect(
                    "Column.array_coltype cannot be None when using `ColumnType::Array`.",
                ) {
                    ColumnType::ID
                    | ColumnType::Int1
                    | ColumnType::UInt1
                    | ColumnType::Int4
                    | ColumnType::UInt4
                    | ColumnType::Int8
                    | ColumnType::UInt8
                    | ColumnType::Int16
                    | ColumnType::UInt16
                    | ColumnType::Timestamp
                    | ColumnType::BlockHeight => "integer",
                    ColumnType::Address
                    | ColumnType::Bytes4
                    | ColumnType::Bytes8
                    | ColumnType::Bytes32
                    | ColumnType::AssetId
                    | ColumnType::ContractId
                    | ColumnType::Salt
                    | ColumnType::MessageId
                    | ColumnType::Charfield
                    | ColumnType::Identity
                    | ColumnType::Bytes64
                    | ColumnType::Signature
                    | ColumnType::Nonce
                    | ColumnType::HexString
                    | ColumnType::TxId
                    | ColumnType::BlockId => "varchar",
                    ColumnType::Blob => "bytea",
                    ColumnType::Json | ColumnType::Virtual => "json",
                    _ => unimplemented!(),
                };

                format!("{t} [1000]")
            }
        }
    }
}

impl SqlFragment for Column {
    /// Return the SQL create statement for a `Column`.
    fn create(&self) -> String {
        let null_frag = if self.nullable { "" } else { "not null" };
        let unique_frag = if self.unique { "unique" } else { "" };
        format!(
            "{} {} {} {}",
            self.name,
            // Will only panic if given an array type
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

/// Type ID used to identify `TypeDefintion`s in the GraphQL schema.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct TypeId {
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
}

impl TypeId {
    /// Create a new `TypeId` from a given `TypeDefinition`.
    pub fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        let type_id = type_id(&parsed.fully_qualified_namespace(), &typ.name.to_string());
        Self {
            id: type_id,
            version: parsed.schema().version().to_string(),
            namespace: parsed.namespace().to_string(),
            identifier: parsed.identifier().to_string(),
            graphql_name: typ.name.to_string(),
            table_name: typ.name.to_string().to_lowercase(),
        }
    }

    /// Create a new `TypeId` from a given `JoinTableItem`.
    pub fn from_join_info(info: JoinTableItem, parsed: &ParsedGraphQLSchema) -> Self {
        let JoinTableItem { table_name, .. } = info;

        let type_id = type_id(&parsed.fully_qualified_namespace(), &table_name);
        Self {
            id: type_id,
            version: parsed.schema().version().to_string(),
            namespace: parsed.namespace().to_string(),
            identifier: parsed.identifier().to_string(),
            // Doesn't matter what this is, but let's use `ID` since all column types
            // on join tables are `ColumnType::ID` for now.
            graphql_name: ColumnType::ID.to_string(),
            table_name,
        }
    }
}

/// `ColumnInfo` is a derived version of `Column` that is only for `queries::columns_get_schema`.
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
    /// Return the unique identifier (UID) of the indexer.
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
#[derive(Debug, Default)]
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
    /// Return the SQL name of the index.
    fn sql_name(&self) -> String {
        format!("{}_{}_idx", &self.table_name, &self.column_name)
    }
}

impl SqlFragment for SqlIndex {
    /// Return the SQL create statement for a `SqlIndex`.
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
                    self.sql_name(),
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
    /// SQL index constraint.
    Index(SqlIndex),

    /// SQL foreign key constraint.
    Fk(ForeignKey),
}

impl SqlFragment for Constraint {
    /// Return the SQL create statement for a `Constraint`.
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

    /// The namespace and the identifier of the indexer joined by an underscore.
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

impl SqlNamed for ForeignKey {
    /// Return the SQL name of the foreign key.
    fn sql_name(&self) -> String {
        format!(
            "fk_{}_{}__{}_{}",
            self.table_name, self.column_name, self.ref_tablename, self.ref_colname
        )
    }
}

impl SqlFragment for ForeignKey {
    /// Return the SQL create statement for the a `ForeignKey`.
    fn create(&self) -> String {
        match self.db_type {
            DbType::Postgres => {
                format!(
                    "ALTER TABLE {}.{} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}({}) ON DELETE {} ON UPDATE {} INITIALLY DEFERRED;",
                    self.namespace,
                    self.table_name,
                    self.sql_name(),
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

/// Nonce used for indexer authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct Nonce {
    /// Unique string used as a nonce payload.
    pub uid: String,

    /// Expiry time of the nonce.
    pub expiry: i64,
}

impl Nonce {
    /// Determine whether or not this nonce has expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        now >= self.expiry
    }
}

#[derive(Default, Debug)]
pub enum TableType {
    /// A table that is used to join two other tables.
    Join,

    /// A normal SQL table with basic constraints.
    #[default]
    Regular,
}

/// SQL database table for a given `GraphRoot` in the database.
#[derive(Default, Debug)]
pub struct Table {
    /// The name of the table.
    name: String,

    /// The namespace of the indexer.
    namespace: String,

    /// The identifier of the indexer.
    identifier: String,

    /// SQL columns associated with this table.
    columns: Vec<Column>,

    /// SQL conswtraints associated with this table.
    constraints: Vec<Constraint>,

    /// How this typedef is persisted to the database.
    persistence: Persistence,

    /// The type of table.
    #[allow(unused)]
    table_type: TableType,
}

impl SqlNamed for Table {
    /// Return the SQL name of the table.
    fn sql_name(&self) -> String {
        self.name.to_string()
    }
}

impl Table {
    pub fn constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }

    pub fn columns(&self) -> &Vec<Column> {
        &self.columns
    }

    /// Create a new `Table` from a given `TypeDefinition`.
    pub fn from_typedef(typ: &TypeDefinition, parsed: &ParsedGraphQLSchema) -> Self {
        let ty_id = type_id(&parsed.fully_qualified_namespace(), &typ.name.to_string());
        match &typ.kind {
            TypeKind::Object(o) => {
                let persistence = if parsed.is_virtual_typedef(&typ.name.to_string()) {
                    Persistence::Virtual
                } else {
                    Persistence::Scalar
                };

                let mut columns = o
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        Column::from_field_def(
                            &f.node,
                            parsed,
                            ty_id,
                            i as i32,
                            persistence,
                        )
                    })
                    .collect::<Vec<Column>>();

                let constraints = o
                    .fields
                    .iter()
                    .filter_map(|f| {

                        // Can't create constraints on array fields.
                        if is_list_type(&f.node) {
                            return None;
                        }

                        let has_unique = f
                            .node
                            .directives
                            .iter()
                            .any(|d| d.node.name.to_string() == "unique");

                        if has_unique {
                            return Some(Constraint::Index(SqlIndex {
                                db_type: DbType::Postgres,
                                table_name: typ.name.to_string().to_lowercase(),
                                namespace: parsed.fully_qualified_namespace(),
                                unique: true,
                                column_name: f.node.name.to_string(),
                                ..SqlIndex::default()
                            }));
                        }

                        let field_typ = f.node.ty.node.to_string().replace(['[', ']', '!'], "");
                        if parsed.is_possible_foreign_key(&field_typ) {
                            let (ref_coltype, ref_colname, ref_tablename) =
                                extract_foreign_key_info(
                                    &f.node,
                                    parsed.field_type_mappings(),
                                );

                            return Some(Constraint::Fk(ForeignKey {
                                db_type: DbType::Postgres,
                                namespace: parsed.fully_qualified_namespace(),
                                table_name: typ.name.to_string().to_lowercase(),
                                column_name: f.node.name.to_string(),
                                ref_tablename,
                                ref_colname,
                                ref_coltype,
                                ..ForeignKey::default()
                            }));
                        }

                        None
                    })
                    .collect::<Vec<Constraint>>();

                // `Object` columns contain the `FtColumn` bytes for each
                // column in the object. This column shouldn't really be public
                columns.push(Column {
                    type_id: ty_id,
                    name: ObjectCol::to_lowercase_string(),
                    graphql_type: "--".to_string(),
                    coltype: ColumnType::Object,
                    position: columns.len() as i32,
                    unique: false,
                    nullable: false,
                    persistence,
                    ..Column::default()
                });

                Self {
                    // TODO: https://github.com/FuelLabs/fuel-indexer/issues/960
                    name: typ.name.to_string().to_lowercase(),
                    namespace: parsed.namespace().to_string(),
                    identifier: parsed.identifier().to_string(),
                    columns,
                    constraints,
                    persistence,
                    table_type: TableType::Regular
                }
            }
            TypeKind::Union(u) => {
                // Since we've already parsed each member of the union, we can
                // just get the set of all member fields and manually build an
                // `TypeDefinition(TypeKind::Object)` from that.
                let union_name = typ.name.to_string();
                let default_ = BTreeMap::<String, String>::new();

                let fields = u
                    .members
                    .iter()
                    .flat_map(|m| {
                        let name = m.node.to_string();
                        parsed
                            .object_field_mappings
                            .get(&name)
                            .unwrap_or(&default_)
                            .iter()
                            .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    })
                    .collect::<LinkedHashSet<(String, String)>>()
                    .iter()
                    .map(|(k, _)| {
                        let fid = field_id(&union_name, k);
                        let f = &parsed
                            .field_defs()
                            .get(&fid)
                            .expect("FieldDefinition not found in parsed schema.");
                        // All fields in a derived union type are nullable, except for
                        // the `ID` field.
                        let mut f = f.0.clone();
                        f.ty.node.nullable =
                            f.name.to_string() != IdCol::to_lowercase_str();
                        Positioned {
                            pos: Pos::default(),
                            node: f,
                        }
                    })
                    .collect::<Vec<Positioned<FieldDefinition>>>();

                let typdef = TypeDefinition {
                    description: None,
                    extend: false,
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new(union_name),
                    },
                    kind: TypeKind::Object(ObjectType {
                        implements: vec![],
                        fields,
                    }),
                    directives: vec![],
                };

                Self::from_typedef(&typdef, parsed)
            }
            _ => unimplemented!("An EnumType TypeDefinition should not have been passed to Table::from_typedef."),
        }
    }

    pub fn from_join_info(item: JoinTableItem, parsed: &ParsedGraphQLSchema) -> Self {
        let JoinTableItem {
            table_name,
            local_table_name,
            column_name,
            ref_table_name,
            ref_column_name,
            ref_column_type,
            ..
        } = item;
        let ty_id = type_id(&parsed.fully_qualified_namespace(), &table_name);
        let columns = vec![
            Column {
                type_id: ty_id,
                name: format!("{local_table_name}_{column_name}"),
                graphql_type: ColumnType::UInt8.to_string(),
                coltype: ColumnType::UInt8,
                position: 0,
                unique: false,
                nullable: false,
                persistence: Persistence::Scalar,
                ..Column::default()
            },
            Column {
                type_id: ty_id,
                name: format!("{ref_table_name}_{ref_column_name}"),
                graphql_type: ColumnType::UInt8.to_string(),
                coltype: ColumnType::UInt8,
                position: 1,
                unique: false,
                nullable: false,
                persistence: Persistence::Scalar,
                ..Column::default()
            },
        ];

        let constraints = vec![
            Constraint::Fk(ForeignKey {
                db_type: DbType::Postgres,
                namespace: parsed.fully_qualified_namespace(),
                table_name: table_name.clone(),
                column_name: format!("{local_table_name}_{column_name}"),
                ref_tablename: ref_table_name.clone(),
                ref_colname: ref_column_name.clone(),
                ref_coltype: ref_column_type.clone(),
                ..ForeignKey::default()
            }),
            Constraint::Fk(ForeignKey {
                db_type: DbType::Postgres,
                namespace: parsed.fully_qualified_namespace(),
                table_name: table_name.clone(),
                column_name: format!("{ref_table_name}_{ref_column_name}"),
                ref_tablename: local_table_name,
                ref_colname: column_name,
                ref_coltype: ref_column_type,
                ..ForeignKey::default()
            }),
        ];

        Self {
            name: table_name,
            namespace: parsed.namespace().to_string(),
            identifier: parsed.identifier().to_string(),
            columns,
            constraints,
            persistence: Persistence::Scalar,
            table_type: TableType::Join,
        }
    }
}

impl SqlFragment for Table {
    /// Return the SQL create statement for a `Table`.
    fn create(&self) -> String {
        match self.persistence {
            Persistence::Scalar => {
                let mut s = format!(
                    "CREATE TABLE {}_{}.{} (\n",
                    self.namespace, self.identifier, self.name
                );
                let cols = self
                    .columns
                    .iter()
                    .map(|c| c.create())
                    .collect::<Vec<String>>()
                    .join(",\n");
                s.push_str(&cols);
                // Remove last ',\n' from last column to avoid syntax error
                let chars = s.chars();

                let mut chars = chars.as_str().to_string();
                chars.push_str("\n);");

                chars
            }
            _ => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use async_graphql_parser::types::BaseType;
    use async_graphql_parser::types::ConstDirective;
    use async_graphql_parser::types::ObjectType;
    use async_graphql_parser::types::Type;
    use fuel_indexer_lib::graphql::GraphQLSchema;
    use fuel_indexer_lib::ExecutionSource;

    #[test]
    fn test_can_create_well_formed_table_and_table_components_when_passed_typedef() {
        let schema = r#"
type Person {
    id: ID!
    name: Charfield! @unique
    age: UInt1!
}"#;

        let fields = [
            ("id", "ID", None),
            (
                "name",
                "Charfield",
                Some(vec![Positioned {
                    pos: Pos::default(),
                    node: ConstDirective {
                        name: Positioned {
                            pos: Pos::default(),
                            node: Name::new("unique"),
                        },
                        arguments: vec![],
                    },
                }]),
            ),
            ("age", "UInt1", None),
        ]
        .iter()
        .map(|(name, typ, directives)| {
            let directives = directives.clone().unwrap_or(vec![]);
            Positioned {
                pos: Pos::default(),
                node: FieldDefinition {
                    description: None,
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new(name),
                    },
                    arguments: vec![],
                    ty: Positioned {
                        pos: Pos::default(),
                        node: Type {
                            base: BaseType::Named(Name::new(typ)),
                            nullable: false,
                        },
                    },
                    directives,
                },
            }
        })
        .collect::<Vec<Positioned<FieldDefinition>>>();
        let typdef = TypeDefinition {
            description: None,
            extend: false,
            name: Positioned {
                pos: Pos::default(),
                node: Name::new("Person"),
            },
            kind: TypeKind::Object(ObjectType {
                implements: vec![],
                fields,
            }),
            directives: vec![],
        };

        let schema = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();

        let table = Table::from_typedef(&typdef, &schema);
        assert_eq!(table.columns().len(), 4);
        assert_eq!(table.constraints().len(), 1);
    }

    #[test]
    fn test_can_create_well_formed_column_from_field_defintion() {
        let schema = r#"
type Person {
    id: ID!
    name: Charfield! @unique
    age: UInt1!
}"#;

        let schema = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();

        let field_def = FieldDefinition {
            description: None,
            name: Positioned {
                pos: Pos::default(),
                node: Name::new("name"),
            },
            arguments: vec![],
            ty: Positioned {
                pos: Pos::default(),
                node: Type {
                    base: BaseType::Named(Name::new("Charfield")),
                    nullable: false,
                },
            },
            directives: vec![Positioned {
                pos: Pos::default(),
                node: ConstDirective {
                    name: Positioned {
                        pos: Pos::default(),
                        node: Name::new("unique"),
                    },
                    arguments: vec![],
                },
            }],
        };

        let type_id = type_id(&schema.fully_qualified_namespace(), "Person");
        let column =
            Column::from_field_def(&field_def, &schema, type_id, 0, Persistence::Scalar);
        assert_eq!(column.graphql_type, "Charfield".to_string());
        assert_eq!(column.coltype, ColumnType::Charfield);
        assert!(column.unique);
        assert!(!column.nullable);
    }
}
