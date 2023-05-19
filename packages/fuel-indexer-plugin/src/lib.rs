#![deny(unused_crate_dependencies)]

#[cfg(feature = "native-execution")]
pub mod native;
pub mod wasm;

extern crate alloc;

pub mod types {
    pub use fuel_indexer_schema::FtColumn;
    pub use fuel_indexer_types::*;

    // These imports are used in the indexer.rs module when iterating over
    // block transactions, in order to cache contract IDs.
    pub use std::collections::{HashMap, HashSet};
}

pub mod utils {
    pub use fuel_indexer_lib::utils::{
        indexer_utils::{
            bytes32_from_inputs, first32_bytes_to_bytes32, first8_bytes_to_u64,
            trim_sized_ascii_string, u64_id, u64_id_from_inputs,
        },
        sha256_digest,
    };
}

pub use bincode;
pub use fuel_indexer_schema::utils::{deserialize, serialize};

// Specifically we import serde here for the `Serialize` and `Deserialize` traits
// else the user would have to explicity import these in their indexer modules.
pub use serde;

pub mod prelude {
    pub use super::{bincode, deserialize, serde, serialize, types::*, utils::*};
}
