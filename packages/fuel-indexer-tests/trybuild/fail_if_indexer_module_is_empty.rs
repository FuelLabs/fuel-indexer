extern crate alloc;
use fuel_indexer_macros::indexer;

#[no_mangle]
fn ff_log_data(_inp: ()) {}

#[indexer(
    manifest = "packages/fuel-indexer-tests/assets/macros/simple_wasm.yaml"
)]
mod indexer {}
