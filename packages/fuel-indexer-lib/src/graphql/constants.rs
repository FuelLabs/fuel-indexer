use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {

    /// Set of internal indexer entities.
    pub static ref INTERNAL_INDEXER_ENTITIES: HashSet<&'static str> = HashSet::from([
        "IndexMetadataEntity",
    ]);

    /// Set of types that implement `AsRef<[u8]>`.
    pub static ref ASREF_BYTE_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "Blob",
        "BlockId",
        "Boolean",
        "Bytes",
        "Bytes20",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "Charfield",
        "ContractId",
        "HexString",
        "Json",
        "MessageId",
        "Virtual",
        "Nonce",
        "Option<Address>",
        "Option<AssetId>",
        "Option<Blob>",
        "Option<BlockId>",
        "Option<Boolean>",
        "Option<Bytes20>",
        "Option<Bytes32>",
        "Option<Bytes4>",
        "Option<Bytes64>",
        "Option<Bytes8>",
        "Option<Bytes>",
        "Option<Charfield>",
        "Option<ContractId>",
        "Option<HexString>",
        "Option<Json>",
        "Option<MessageId>",
        "Option<Virtual>",
        "Option<Nonce>",
        "Option<Salt>",
        "Option<Signature>",
        "Option<TxId>",
        "Salt",
        "Signature",
        "TxId",
    ]);

    /// Set of external types that do not implement `AsRef<[u8]>`.
    pub static ref EXTERNAL_FIELD_TYPES: HashSet<&'static str> = HashSet::from([
        "Identity",
        "Option<Identity>",
        "Option<Tai64Timestamp>",
        "Tai64Timestamp",
    ]);

    /// Set of field types that are currently unable to be used as a digest for SHA-256 hashing.
    pub static ref NON_DIGESTIBLE_FIELD_TYPES: HashSet<&'static str> = HashSet::from([
        "Boolean",
        "Identity"
    ]);


    /// Set of types that should be copied instead of referenced.
    pub static ref COPY_TYPES: HashSet<&'static str> = HashSet::from([
        "Blob",
        "Charfield",
        "HexString",
        "Identity",
        "Json",
        "Virtual",
        "Option<Blob>",
        "Option<Charfield>",
        "Option<HexString>",
        "Option<Identity>",
        "Option<Json>",
        "Option<Virtual>",
    ]);
}
