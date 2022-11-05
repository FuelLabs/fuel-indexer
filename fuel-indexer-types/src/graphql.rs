use crate::{Bytes32, GraphQlEntity};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Jsonb(pub String);

pub struct IndexMetadata {
    pub id: Bytes32,
    pub block_height: u64,
    pub time: u64,
}

impl GraphQlEntity for IndexMetadata {
    fn schema_fragment() -> &'static str {
        r#"

type IndexMetadataEntity {
    id: Bytes32! @unique
    block_height: UInt8!
    time: Int8!
}
"#
    }
}
