use crate::scalar::ID;
use serde::{Deserialize, Serialize};

/// Native GraphQL `TypeDefinition` used to keep track of chain metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexMetadata {
    /// Metadata identifier.
    pub id: ID,

    /// Time of metadata.
    pub time: u64,

    /// Block height of metadata.
    pub block_height: u32,

    /// Block ID of metadata.
    pub block_id: String,
}

impl IndexMetadata {
    /// Return the GraphQL schema fragment for the `IndexMetadata` type.
    ///
    /// The structure of this fragment should always match `fuel_indexer_types::IndexMetadata`.
    pub fn schema_fragment() -> &'static str {
        r#"

type IndexMetadataEntity @entity {
    id: ID!
    time: UInt8!
    block_height: UInt4!
    block_id: Bytes32!
}
"#
    }
}
