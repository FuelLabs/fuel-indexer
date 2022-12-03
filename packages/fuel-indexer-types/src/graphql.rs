use fuel_types::Bytes32;

pub trait GraphqlObject {
    fn schema_fragment() -> &'static str;
}

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
