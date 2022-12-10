extern crate alloc;
use fuel_indexer::prelude::*;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "examples/hello-world-native/hello_index_native.manifest.yaml")]
mod hello_world_native {
    async fn foo(_block_data: BlockData) {
        Logger::info("I'm in native execution.");
    }
}
