#[cfg(feature = "native-execution")]
pub mod native;

pub mod wasm;

extern crate alloc;

pub mod types {
    pub use fuel_indexer_schema::FtColumn;
    pub use fuel_indexer_types::*;
}

pub mod utils {
    pub use fuel_indexer_lib::utils::{
        index_utils::{
            bytes32_from_inputs, first32_bytes_to_bytes32, first8_bytes_to_u64,
            trim_sized_ascii_string, u64_id, u64_id_from_inputs,
        },
        sha256_digest,
    };
}

pub mod prelude {
    pub use super::types::*;

    pub use super::utils::*;
}
