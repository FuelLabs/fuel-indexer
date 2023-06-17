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
    pub static ref NONDIGESTIBLE_FIELD_TYPES: HashSet<&'static str> = HashSet::from([
        "Boolean",
        "Identity"
    ]);

    pub static ref FUEL_PRIMITIVES: HashSet<&'static str> = HashSet::from([
        "BlockData",
        "Call",
        "Log",
        "LogData",
        "MessageOut",
        "Panic",
        "Return",
        "Revert",
        "ScriptResult",
        "Transfer",
        "TransferOut",
    ]);
    pub static ref DISALLOWED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from([]);
    pub static ref IGNORED_ABI_JSON_TYPES: HashSet<&'static str> =
        HashSet::from(["()", "struct Vec"]);
    pub static ref GENERIC_TYPES: HashSet<&'static str> = HashSet::from([
        "generic T",
        "raw untyped ptr",
        "struct RawVec",
        "struct Vec"
    ]);
    pub static ref FUEL_RECEIPT_TYPES: HashSet<&'static str> = HashSet::from([
        "Call",
        "Log",
        "LogData",
        "MessageOut",
        "Panic",
        "Return",
        "Revert",
        "ScriptResult",
        "Transfer",
        "TransferOut",
    ]);
    pub static ref RUST_PRIMITIVES: HashSet<&'static str> =
        HashSet::from(["u8", "u16", "u32", "u64", "bool", "String"]);

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

    pub static ref DISALLOWED_OBJECT_NAMES: HashSet<&'static str> = HashSet::from([
        // Scalars.
        "Address",
        "AssetId",
        "Blob",
        "BlockHeight",
        "BlockId",
        "Boolean",
        "Bytes",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "Charfield",
        "Color",
        "ContractId",
        "HexString",
        "ID",
        "Identity",
        "Int1",
        "Int16",
        "Int4",
        "Int8",
        "Json",
        "MessageId",
        "Nonce",
        "Virtual",
        "Salt",
        "Signature",
        "Tai64Timestamp",
        "Timestamp",
        "TxId",
        "UInt1",
        "UInt16",
        "UInt4",
        "UInt8",

        // Imports for transaction fields.
        // https://github.com/FuelLabs/fuel-indexer/issues/286
        "BlockData",
        "BytecodeLength",
        "BytecodeWitnessIndex",
        "FieldTxPointer",
        "GasLimit",
        "GasPrice",
        "Inputs",
        "Log",
        "LogData",
        "Maturity",
        "MessageId",
        "Outputs",
        "ReceiptsRoot",
        "Script",
        "ScriptData",
        "ScriptResult",
        "StorageSlots",
        "TransactionData",
        "Transfer",
        "TransferOut",
        "TxFieldSalt",
        "TxFieldScript",
        "TxId",
        "Witnesses",
    ]);
}
