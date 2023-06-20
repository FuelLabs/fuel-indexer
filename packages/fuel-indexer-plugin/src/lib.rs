#![deny(unused_crate_dependencies)]

#[cfg(feature = "native-execution")]
pub mod native;
pub mod wasm;

extern crate alloc;

pub mod types {
    pub use fuel_indexer_schema::FtColumn;
    pub use fuel_indexer_types::fuel::{BlockData, TxId};

    // Traits needed to access client type fields. Could also include this as a sub-module
    // of `fuel_indexer_types::fuel`.
    pub use fuel_indexer_types::fuel::field::*;
    pub use fuel_indexer_types::{fuel, prelude::*};

    // These imports are used in the indexer.rs module when iterating over
    // block transactions, in order to cache contract IDs.
    pub use std::collections::{HashMap, HashSet};
}

pub mod utils {
    pub use fuel_indexer_lib::utils::sha256_digest;
}

pub use bincode;
pub use fuel_indexer_lib::utils::{deserialize, serialize};

// Specifically we import `serde` here for the `Serialize` and `Deserialize` traits
// else the user would have to explicity import these in their indexer modules.
pub use serde;

// We import `serde_json` for the `From<T> for Json` in the `fuel-indexer-macro/schema` module.
pub use serde_json;

pub mod prelude {
    pub use super::{
        bincode, deserialize, serde, serde_json, serialize, types::*, utils::*,
    };
    pub use crate::{debug, error, info, trace, warn};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        Logger::error(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        Logger::warn(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        Logger::info(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        Logger::debug(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        Logger::trace(&format!($($arg)*))
    }};
}
