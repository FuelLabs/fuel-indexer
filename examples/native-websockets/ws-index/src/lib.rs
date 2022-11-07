extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "examples/native-websockets/manifest.yaml")]
pub mod native_websockets {

    fn foobar(_block_data: BlockData) {
        NativeLogger::info("I am in foobar");
    }
}
