pub mod constants;
pub mod parser;
pub mod types;
pub mod validator;

pub use parser::{ParsedError, ParsedGraphQLSchema};
pub use validator::GraphQLSchemaValidator;

use async_graphql_parser::types::FieldDefinition;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use types::IdCol;

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
    /// Return the GraphQL schema fragment for the `IndexMetadata` type.
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
    /// Create a new `GraphQLSchema` from raw GraphQL content.
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

/// Given a `FieldDefinition` that is a possible foreign key (according to `ParsedGraphQLSchema`),
/// return the column type, column name, and table name of the foreign key.
///
/// We pass `ParsedGraphQLSchema::field_type_mappings` here instead of the full `ParsedGraphQLSchema`
/// because when using `extract_foreign_key_info` in `ParsedGraphQLSchema` we don't have access to the
/// fully parsed `ParsedGraphQLSchema` yet.
pub fn extract_foreign_key_info(
    f: &FieldDefinition,
    field_type_mappings: &HashMap<String, String>,
) -> (String, String, String) {
    let (ref_coltype, ref_colname, ref_tablename) = f
        .directives
        .iter()
        .find(|d| d.node.name.to_string() == "join")
        .map(|d| {
            let typdef_name = f.ty.to_string().replace('!', "");
            let ref_field_name = d.clone().node.arguments.pop().unwrap().1.to_string();
            let fk_fid = field_id(&typdef_name, &ref_field_name);
            let fk_field_type = field_type_mappings.get(&fk_fid).unwrap().to_string();

            (
                fk_field_type.replace('!', ""),
                ref_field_name,
                typdef_name.to_lowercase(),
            )
        })
        .unwrap_or((
            IdCol::to_uppercase_string(),
            IdCol::to_lowercase_string(),
            f.ty.to_string().replace('!', "").to_lowercase(),
        ));

    (ref_coltype, ref_colname, ref_tablename)
}

/// Return a fully qualified name for a given `FieldDefinition` on a given `TypeDefinition`.
pub fn field_id(typdef_name: &str, field_name: &str) -> String {
    format!("{typdef_name}.{field_name}")
}
