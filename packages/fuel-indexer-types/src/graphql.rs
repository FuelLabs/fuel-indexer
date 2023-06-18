use fuel_types::Bytes32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub trait GraphqlObject {
    fn schema_fragment() -> &'static str;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexMetadata {
    pub id: u64,
    pub block_height: u32,
    pub time: u64,
}

impl GraphqlObject for IndexMetadata {
    fn schema_fragment() -> &'static str {
        r#"

type IndexMetadataEntity {
    id: ID!
    time: UInt8!
    block_height: UInt4!
}
"#
    }
}

fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}

fn inject_native_entities_into_schema(schema: &str) -> String {
    format!("{}{}", schema, IndexMetadata::schema_fragment())
}

pub struct GraphQLSchema {
    schema: String,
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
