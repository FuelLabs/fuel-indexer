use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

lazy_static! {

    /// Set of internal indexer entities.
    pub static ref INTERNAL_INDEXER_ENTITIES: HashSet<&'static str> = HashSet::from([
        "IndexMetadataEntity",
    ]);

    /// Set of types that implement `AsRef<[u8]>`.
    pub static ref ASREF_BYTE_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "Bytes",
        "Boolean",
        "Bytes",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "String",
        "ContractId",
        "Json",
        "Option<Address>",
        "Option<AssetId>",
        "Option<Bytes>",
        "Option<Boolean>",
        "Option<Bytes20>",
        "Option<Bytes32>",
        "Option<Bytes4>",
        "Option<Bytes64>",
        "Option<Bytes8>",
        "Option<Bytes>",
        "Option<String>",
        "Option<ContractId>",
        "Option<Json>",
    ]);

    /// Set of external types that do not implement `AsRef<[u8]>`.
    pub static ref EXTERNAL_FIELD_TYPES: HashSet<&'static str> = HashSet::from([
        "Identity",
        "Option<Identity>",
    ]);

    /// Set of field types that are currently unable to be used as a digest for SHA-256 hashing.
    pub static ref NON_DIGESTIBLE_FIELD_TYPES: HashSet<&'static str> = HashSet::from([
        "Boolean",
        "Identity"
    ]);


    /// Set of types that should be copied instead of referenced.
    pub static ref COPY_TYPES: HashSet<&'static str> = HashSet::from([
        "Bytes",
        "String",
        "Identity",
        "Json",
        "Option<Bytes>",
        "Option<String>",
        "Option<Identity>",
        "Option<Json>",
    ]);

    /// The mapping of Sway types to GraphQL types used in automatic GraphQL schema generation.
    pub static ref ABI_TYPE_MAP: HashMap<&'static str, &'static str> = HashMap::from_iter([
        ("u128", "U128"),
        ("u64", "U64"),
        ("u32", "U32"),
        ("u8", "U8"),
        ("i128", "I128"),
        ("i64", "I64"),
        ("i32", "I32"),
        ("i8", "I8"),
        ("bool", "Boolean"),
        ("u8[64]", "Bytes64"),
        ("u8[32]", "Bytes32"),
        ("u8[8]", "Bytes8"),
        ("u8[4]", "Bytes4"),
        ("Vec<u8>", "Bytes"),
        ("SizedAsciiString<64>", "ID"),
        ("String", "String"),
        ("str[32]", "Bytes32"),
        ("str[64]", "Bytes64"),
    ]);
}
