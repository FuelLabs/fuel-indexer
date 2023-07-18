extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "examples/scalar-types/scalar-types/scalar_types.manifest.yaml")]
pub mod scalar_types_index_mod {

    fn scalar_types_handler(block_data: BlockData) {}
}
