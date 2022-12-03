extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "examples/native-execution/native_index.manifest.yaml")]
pub mod native_execution {

    fn handle_event_using_native(block: BlockData) {
        NativeLogger::info("I am in native execution");
    }
}
