pub mod parser;

pub use parser::{ParsedError, ParsedGraphQLSchema};

use async_graphql_parser::types::{Directive, FieldDefinition};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const BASE_SCHEMA: &str = include_str!("./base.graphql");

/// Derive version of GraphQL schema content via SHA256.
pub fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}

/// Native GraphQL `TypeDefinition` used to keep track of chain metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexMetadata {
    /// Metadata identifier.
    pub id: u64,

    /// Block height of metadata.
    pub block_height: u32,

    /// Time of metadata.
    pub time: u64,
}

impl IndexMetadata {
    pub fn schema_fragment() -> &'static str {
        r#"

type IndexMetadataEntity {
    id: ID!
    time: UInt8!
    block_height: UInt4!
}
"#
    }
}

/// Inject native entities into the GraphQL schema.
fn inject_native_entities_into_schema(schema: &str) -> String {
    format!("{}{}", schema, IndexMetadata::schema_fragment())
}

/// Wrapper for GraphQL schema content.
#[derive(Default, Debug, Clone)]
pub struct GraphQLSchema {
    /// Raw GraphQL schema content.
    schema: String,

    /// Version of the schema.
    version: String,
}

impl From<String> for GraphQLSchema {
    fn from(s: String) -> Self {
        let schema = inject_native_entities_into_schema(&s);
        let version = schema_version(&s);
        Self { schema, version }
    }
}

impl GraphQLSchema {
    pub fn new(content: String) -> Self {
        let schema = inject_native_entities_into_schema(&content);
        let version = schema_version(&schema);
        Self { schema, version }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

impl ToString for GraphQLSchema {
    fn to_string(&self) -> String {
        self.schema.clone()
    }
}
