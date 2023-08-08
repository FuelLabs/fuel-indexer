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
        "Option<HexString>",
        "Option<Json>",
        "Option<MessageId>",
        "Option<Nonce>",
        "Option<Salt>",
        "Option<Signature>",
        "Option<String64>",
        "Option<TxId>",
        "Option<Virtual>",
        "Salt",
        "Signature",
        "String64",
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

    /// ABI types not allowed in the contract ABI.
    pub static ref DISALLOWED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from([]);

    /// Sway ABI types we don't support and won't in the near future.
    pub static ref IGNORED_ABI_JSON_TYPES: HashSet<&'static str> =
        HashSet::from(["()", "struct Vec"]);

    /// Generic Sway ABI types.
    pub static ref GENERIC_TYPES: HashSet<&'static str> = HashSet::from([
        "generic T",
        "raw untyped ptr",
        "struct RawVec",
        "struct Vec"
    ]);

    /// Fuel VM receipt-related types.
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
        "Mint",
        "Burn",
    ]);

    /// Set of Rust primitive types.
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
        "Vec<FtColumn>"
    ]);

    /// Type names that are not allowed in GraphQL schema.
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
        "Salt",
        "Signature",
        "String64",
        "Tai64Timestamp",
        "Timestamp",
        "TxId",
        "UInt1",
        "UInt16",
        "UInt4",
        "UInt8",
        "Virtual",

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
