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
        "Option<Bytes>",
        "Option<Bytes32>",
        "Option<Bytes4>",
        "Option<Bytes64>",
        "Option<Bytes8>",
        "Option<String>",
        "Option<ContractId>",
        "Option<Json>",
        "Option<Salt>",
        "Option<Signature>",
        "Option<UID>",
        "UID",
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
        HashSet::from(["()"]);

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
        "Bytes",
        "ID",
        "Identity",
        "Json",
        "Option<Bytes>",
        "Option<ID>",
        "Option<Identity>",
        "Option<Json>",
        "Option<UID>",
        "Option<String>",
        "UID",
        "Vec<FtColumn>",
        "String",
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
        "Boolean",
        "Bytes",
        "Bytes",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "ContractId",
        "I128",
        "I32",
        "I64",
        "I8",
        "ID",
        "Identity",
        "Json",
        "U128",
        "U32",
        "U64",
        "U8",
        "UID",
        "String",

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
    pub static ref UNSUPPORTED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from(["Vec"]);

    /// Generic Sway ABI types.
    pub static ref IGNORED_GENERIC_METADATA: HashSet<&'static str> = HashSet::from([
        "generic T",
        "generic E",
        "raw untyped ptr",
        "struct RawVec",
        "struct RawBytes",
        "struct Bytes",
        "enum Result"
    ]);

    pub static ref GENERIC_STRUCTS: HashSet<&'static str> = HashSet::from([
        "Vec",
        "Option"
    ]);
}
