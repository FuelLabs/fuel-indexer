pub mod parser;

pub use parser::{ParsedError, ParsedGraphQLSchema};

use async_graphql_parser::types::FieldDefinition;
use sha2::{Digest, Sha256};

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");

/// Remove special chars from GraphQL field type name.
pub fn normalize_field_type_name(name: &str) -> String {
    name.replace('!', "")
}

/// Convert GraphQL field type name to SQL table name.
pub fn field_type_table_name(f: &FieldDefinition) -> String {
    normalize_field_type_name(&f.ty.to_string()).to_lowercase()
}

pub fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}
