use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    pub static ref FUEL_PRIMITIVES: HashSet<&'static str> = HashSet::from([
        "BlockData",
        "Log",
        "LogData",
        "MessageOut",
        "Return",
        "ScriptResult",
        "Transfer",
        "TransferOut",
    ]);
    pub static ref FUEL_PRIMITIVES_NAMESPACED: HashSet<&'static str> = HashSet::from([
        "abi :: BlockData",
        "abi :: Log",
        "abi :: LogData",
        "abi :: MessageOut",
        "abi :: Return",
        "abi :: ScriptResult",
        "abi :: Transfer",
        "abi :: TransferOut",
    ]);
    pub static ref DISALLOWED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from([]);
    pub static ref IGNORED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from(["()"]);
    pub static ref VEC_GENERIC_TYPES: HashSet<&'static str> = HashSet::from([
        "generic T",
        "raw untyped ptr",
        "struct RawVec",
        "struct Vec"
    ]);
    pub static ref FUEL_RECEIPT_TYPES: HashSet<&'static str> = HashSet::from([
        "Log",
        "LogData",
        "MessageOut",
        "Return",
        "ScriptResult",
        "Transfer",
        "TransferOut",
    ]);
    pub static ref RUST_PRIMITIVES: HashSet<&'static str> =
        HashSet::from(["u8", "u16", "u32", "u64", "bool", "String"]);
}
