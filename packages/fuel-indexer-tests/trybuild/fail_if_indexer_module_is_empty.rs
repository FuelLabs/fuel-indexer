use fuel_indexer_utils::prelude::*;

#[no_mangle]
fn ff_log_data(_inp: ()) {}

#[indexer(manifest = "packages/fuel-indexer-tests/trybuild/simple_wasm.yaml")]
mod indexer {}
