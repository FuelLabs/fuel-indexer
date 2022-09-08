extern crate alloc;
use fuel_indexer_macros::indexer;

#[no_mangle]
fn ff_log_data(_inp: ()) {}


#[indexer(
    abi = "./test_data/contracts-abi.json",
    namespace = "test_namespace",
    schema = "./test_data/schema.graphql",
)]
mod indexer {
}
