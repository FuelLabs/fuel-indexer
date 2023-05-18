pub trait GraphqlObject {
    fn schema_fragment() -> &'static str;
}

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
    time: Int8!
}
"#
    }
}
