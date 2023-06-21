pub mod parser;

pub use parser::{ParsedError, ParsedGraphQLSchema};

use async_graphql_parser::types::{Directive, FieldDefinition};
use sha2::{Digest, Sha256};

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");

pub fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}
