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
        "Bytes",
        "Json",
        "MessageId",
        "Nonce",
        "Option<Address>",
        "Option<AssetId>",
        "Option<Blob>",
        "Option<BlockId>",
        "Option<Boolean>",
        "Option<Bytes>",
        "Option<Bytes20>",
        "Option<Bytes32>",
        "Option<Bytes4>",
        "Option<Bytes64>",
        "Option<Bytes8>",
        "Option<Charfield>",
        "Option<ContractId>",
        "Option<Bytes>",
        "Option<Json>",
        "Option<MessageId>",
        "Option<Nonce>",
        "Option<Salt>",
        "Option<Signature>",
        "Option<UID>",
        "Option<TxId>",
        "Option<Virtual>",
        "Salt",
        "Signature",
        "UID",
        "TxId",
        "Virtual",
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

    /// Sway ABI types we don't support and won't in the near future.
    pub static ref IGNORED_ABI_JSON_TYPES: HashSet<&'static str> =
        HashSet::from(["()", "struct Vec"]);

    /// Fuel VM receipt-related types.
    pub static ref FUEL_RECEIPT_TYPES: HashSet<&'static str> = HashSet::from([
        "Burn",
        "Call",
        "Log",
        "LogData",
        "MessageOut",
        "Mint",
        "Panic",
        "Return",
        "Revert",
        "ScriptResult",
        "Transfer",
        "TransferOut",
    ]);

    /// Set of types that should be copied instead of referenced.
    pub static ref COPY_TYPES: HashSet<&'static str> = HashSet::from([
        "Blob",
        "Charfield",
        "Bytes",
        "ID",
        "Identity",
        "Json",
        "Option<Blob>",
        "Option<Charfield>",
        "Option<Bytes>",
        "Option<ID>",
        "Option<Identity>",
        "Option<Json>",
        "Option<UID>",
        "Option<Virtual>",
        "UID",
        "Vec<FtColumn>",
        "Virtual",
    ]);

    /// Fuel-specific receipt-related type names.
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
        "Mint",
        "Burn",
    ]);

    /// Type names that are not allowed in GraphQL schema.
    pub static ref RESERVED_TYPEDEF_NAMES: HashSet<&'static str> = HashSet::from([
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
        "Bytes",
        "ID",
        "Identity",
        "I8",
        "I128",
        "I32",
        "I64",
        "Json",
        "MessageId",
        "Nonce",
        "Salt",
        "Signature",
        "Tai64Timestamp",
        "Timestamp",
        "TxId",
        "UID",
        "U8",
        "U128",
        "U32",
        "U64",
        "Virtual",

        // Imports for transaction fields.
        // https://github.com/FuelLabs/fuel-indexer/issues/286
        "BlockData",
        "Burn",
        "BytecodeLength",
        "BytecodeWitnessIndex",
        "Call",
        "FieldTxPointer",
        "GasLimit",
        "GasPrice",
        "Inputs",
        "Log",
        "Log",
        "LogData",
        "LogData",
        "Maturity",
        "MessageId",
        "MessageOut",
        "Mint",
        "Outputs",
        "Panic",
        "ReceiptsRoot",
        "Return",
        "Revert",
        "Script",
        "ScriptData",
        "ScriptResult",
        "ScriptResult",
        "StorageSlots",
        "TransactionData",
        "Transfer",
        "Transfer",
        "TransferOut",
        "TransferOut",
        "TxFieldSalt",
        "TxFieldScript",
        "TxId",
        "Witnesses",
    ]);


    /// ABI types not allowed in the contract ABI.
    pub static ref DISALLOWED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from([]);

    /// Generic Sway ABI types.
    pub static ref GENERIC_TYPES: HashSet<&'static str> = HashSet::from([
        "generic T",
        "raw untyped ptr",
        "struct RawVec",
    ]);

    pub static ref COLLECTION_TYPES: HashSet<&'static str> = HashSet::from([
        "Vec",
    ]);

    /// Set of Rust primitive types.
    pub static ref RUST_PRIMITIVES: HashSet<&'static str> =
        HashSet::from(["u8", "u16", "u32", "u64", "bool", "String"]);

}
