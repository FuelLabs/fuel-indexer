use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    pub static ref FUEL_PRIMITIVES: HashSet<&'static str> = HashSet::from([
        "Block",
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
}
