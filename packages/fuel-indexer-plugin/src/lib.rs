#[cfg(feature = "native-execution")]
pub mod native;

pub mod wasm;

extern crate alloc;

pub mod types {
    pub use fuel_indexer_schema::FtColumn;
    pub use fuel_indexer_types::*;
}

pub mod utils {
    pub use fuel_indexer_lib::utils::sha256_digest;
}

pub mod prelude {
    pub use super::types::*;

    pub use super::utils::*;
}
