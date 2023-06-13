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

    // This is needed to allow for auto ID calculations in the event that a timestamp is
    // marked optional; we provide a default value but it needs to be wrapped in a Tai64 type.
    pub use tai64::Tai64;
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
}
