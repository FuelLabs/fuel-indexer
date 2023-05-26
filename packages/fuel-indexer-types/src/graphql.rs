use fuel_types::Bytes32;
use serde::{Deserialize, Serialize};

pub trait GraphqlObject {
    fn schema_fragment() -> &'static str;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexMetadata {
    pub id: Bytes32,
    pub block_height: u64,
    pub time: u64,
}

impl GraphqlObject for IndexMetadata {
    fn schema_fragment() -> &'static str {
        r#"

type IndexMetadataEntity {
    id: ID!
    time: Int8!
}
"#
    }
}
