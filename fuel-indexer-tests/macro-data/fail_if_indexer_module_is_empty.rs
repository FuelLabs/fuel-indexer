extern crate alloc;
use fuel_indexer_macros::indexer;

#[no_mangle]
fn ff_log_data(_inp: ()) {}


#[indexer(
    abi = "./../examples/simple-wasm/contracts/out/debug/contracts-abi.json",
    namespace = "test_namespace",
    identifier = "index1",
    schema = "./../examples/simple-wasm/schema/schema.graphql",
)]
mod indexer {
}
