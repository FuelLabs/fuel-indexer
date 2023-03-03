#![deny(unused_crate_dependencies)]

use crate::directives::IndexMethod;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::string::ToString;
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
    pub schema_identifier: String,
    pub query: String,
    pub schema: String,
}

#[derive(Debug)]
pub struct NewGraphRoot {
    pub version: String,
    pub schema_name: String,
    pub schema_identifier: String,
    pub query: String,
    pub schema: String,
}

#[derive(Debug)]
pub struct TypeId {
    pub id: i64,
    pub schema_version: String,
    pub schema_name: String,
    pub schema_identifier: String,
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
            ColumnType::Int16 => "numeric",
            ColumnType::UInt4 => "integer",
            ColumnType::UInt8 => "bigint",
            ColumnType::UInt16 => "numeric",
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
    UInt16 = 21,
    Int16 = 22,
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
            _ => panic!("Invalid column type: '{name}'"),
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
}

impl DbType {
    pub fn table_name(&self, namespace: &str, table_name: &str) -> String {
        match self {
            DbType::Postgres => format!("{namespace}.{table_name}"),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueryElement {
    Field { key: String, value: String },
    ObjectOpeningBoundary { key: String },
    ObjectClosingBoundary,
}

// TODO: Adjust filter to allow for more complex filtering
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QueryFilter {
    pub key: String,
    pub relation: String,
    pub value: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct JoinCondition {
    pub referencing_key_table: String,
    pub referencing_key_col: String,
    pub primary_key_table: String,
    pub primary_key_col: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QueryJoinNode {
    pub dependencies: HashMap<String, JoinCondition>,
    pub dependents: HashMap<String, JoinCondition>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UserQuery {
    pub elements: Vec<QueryElement>,
    pub joins: HashMap<String, QueryJoinNode>,
    pub namespace_identifier: String,
    pub entity_name: String,
    pub filters: Vec<QueryFilter>,
}

impl UserQuery {
    // TODO: Add proper parsing for filtering
    pub fn to_sql(&mut self, db_type: &DbType) -> String {
        // Different database solutions have unique ways of
        // constructing JSON-formatted queries and results.
        match db_type {
            DbType::Postgres => {
                let elements = self.parse_query_elements(db_type);

                let _filters: Vec<String> = self
                    .filters
                    .iter()
                    .map(|f| format!("{} {} {}", f.key, f.relation, f.value))
                    .collect();

                let elements_string = elements.join("");

                let sorted_joins = self.get_topologically_sorted_joins();

                let mut last_seen_primary_key_table = "".to_string();
                let mut joins: Vec<String> = Vec::new();

                for sj in sorted_joins {
                    if sj.primary_key_table == last_seen_primary_key_table {
                        if let Some(elem) = joins.last_mut() {
                            let join_condition = format!(
                                "{}.{} = {}.{}",
                                sj.referencing_key_table,
                                sj.referencing_key_col,
                                sj.primary_key_table,
                                sj.primary_key_col
                            );
                            *elem = format!("{} AND {}", elem, join_condition)
                        }
                    } else {
                        joins.push(format!(
                            "INNER JOIN {} ON {}.{} = {}.{}",
                            sj.primary_key_table,
                            sj.referencing_key_table,
                            sj.referencing_key_col,
                            sj.primary_key_table,
                            sj.primary_key_col
                        ));
                        last_seen_primary_key_table = sj.primary_key_table;
                    }
                }

                format!(
                    "SELECT json_build_object({}) FROM {}.{} {}",
                    elements_string,
                    self.namespace_identifier,
                    self.entity_name,
                    joins.join(" ")
                )
            }
        }
    }

    fn parse_query_elements(&self, db_type: &DbType) -> Vec<String> {
        let mut peekable_elements = self.elements.iter().peekable();

        let mut elements = Vec::new();

        match db_type {
            DbType::Postgres => {
                while let Some(e) = peekable_elements.next() {
                    match e {
                        // Set the key for this JSON element to the name of the entity field
                        // and the value to the corresponding database table so that it can
                        // be successfully retrieved.
                        QueryElement::Field { key, value } => {
                            elements.push(format!("'{key}', {value}"));

                            // If the next element is not a closing boundary, then a comma should
                            // be added so that the resultant SQL query can be properly constructed.
                            if let Some(next_element) = peekable_elements.peek() {
                                match next_element {
                                    QueryElement::Field { .. }
                                    | QueryElement::ObjectOpeningBoundary { .. } => {
                                        elements.push(", ".to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // Set a nested JSON object as the value for this entity field.
                        QueryElement::ObjectOpeningBoundary { key } => {
                            elements.push(format!("'{key}', json_build_object("))
                        }

                        QueryElement::ObjectClosingBoundary => {
                            elements.push(")".to_string());

                            if let Some(next_element) = peekable_elements.peek() {
                                match next_element {
                                    QueryElement::Field { .. }
                                    | QueryElement::ObjectOpeningBoundary { .. } => {
                                        elements.push(", ".to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        elements
    }

    fn get_topologically_sorted_joins(&mut self) -> Vec<JoinCondition> {
        let mut yet_to_process =
            self.joins.clone().into_keys().collect::<HashSet<String>>();
        let mut start_nodes: Vec<String> = self
            .joins
            .iter()
            .filter(|(_k, v)| v.dependencies.is_empty())
            .map(|(k, _v)| k.clone())
            .collect();

        let mut sorted_joins: Vec<JoinCondition> = Vec::new();

        while let Some(current_node) = start_nodes.pop() {
            if let Some(node) = self.joins.get_mut(&current_node) {
                for (dependent_node, _) in node.clone().dependents.iter() {
                    if let Some(or) = self.joins.get_mut(dependent_node) {
                        if let Some(dependency) = or.dependencies.remove(&current_node) {
                            sorted_joins.push(dependency);
                            if or.dependencies.is_empty() {
                                start_nodes.push(dependent_node.clone());
                            }
                        }
                    }
                }
            }

            yet_to_process.remove(&current_node);
        }

        sorted_joins.into_iter().rev().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_query_parse_query_elements() {
        let elements = vec![
            QueryElement::Field {
                key: "flat_field_key".to_string(),
                value: "flat_value".to_string(),
            },
            QueryElement::ObjectOpeningBoundary {
                key: "nested_object_key".to_string(),
            },
            QueryElement::Field {
                key: "nested_field_key".to_string(),
                value: "nested_field_value".to_string(),
            },
            QueryElement::ObjectClosingBoundary,
        ];
        let uq = UserQuery {
            elements,
            joins: HashMap::new(),
            namespace_identifier: "".to_string(),
            entity_name: "".to_string(),
            filters: Vec::new(),
        };

        let expected = vec![
            "'flat_field_key', flat_value".to_string(),
            ", ".to_string(),
            "'nested_object_key', json_build_object(".to_string(),
            "'nested_field_key', nested_field_value".to_string(),
            ")".to_string(),
        ];

        assert_eq!(expected, uq.parse_query_elements(&DbType::Postgres));
    }

    #[test]
    fn test_user_query_to_sql() {
        let elements = vec![
            QueryElement::Field {
                key: "hash".to_string(),
                value: "name_ident.block.hash".to_string(),
            },
            QueryElement::ObjectOpeningBoundary {
                key: "tx".to_string(),
            },
            QueryElement::Field {
                key: "hash".to_string(),
                value: "name_ident.tx.hash".to_string(),
            },
            QueryElement::ObjectClosingBoundary,
            QueryElement::Field {
                key: "height".to_string(),
                value: "name_ident.block.height".to_string(),
            },
        ];

        let mut uq = UserQuery {
            elements,
            joins: HashMap::from([
                (
                    "name_ident.block".to_string(),
                    QueryJoinNode {
                        dependencies: HashMap::new(),
                        dependents: HashMap::from([(
                            "name_ident.tx".to_string(),
                            JoinCondition {
                                referencing_key_table: "name_ident.tx".to_string(),
                                referencing_key_col: "block".to_string(),
                                primary_key_table: "name_ident.block".to_string(),
                                primary_key_col: "id".to_string(),
                            },
                        )]),
                    },
                ),
                (
                    "name_ident.tx".to_string(),
                    QueryJoinNode {
                        dependents: HashMap::new(),
                        dependencies: HashMap::from([(
                            "name_ident.block".to_string(),
                            JoinCondition {
                                referencing_key_table: "name_ident.tx".to_string(),
                                referencing_key_col: "block".to_string(),
                                primary_key_table: "name_ident.block".to_string(),
                                primary_key_col: "id".to_string(),
                            },
                        )]),
                    },
                ),
            ]),
            namespace_identifier: "name_ident".to_string(),
            entity_name: "entity_name".to_string(),
            filters: vec![QueryFilter {
                key: "a".to_string(),
                relation: "=".to_string(),
                value: "123".to_string(),
            }],
        };

        let expected = "SELECT json_build_object('hash', name_ident.block.hash, 'tx', json_build_object('hash', name_ident.tx.hash), 'height', name_ident.block.height) FROM name_ident.entity_name INNER JOIN name_ident.block ON name_ident.tx.block = name_ident.block.id"
            .to_string();
        assert_eq!(expected, uq.to_sql(&DbType::Postgres));
    }
}
