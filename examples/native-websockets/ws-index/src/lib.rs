extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "examples/native-websockets/manifest.yaml")]
pub mod native_websockets {

    async fn foobar(event: fuel::Log) {
        Logger::info("Received log");
    }
}
