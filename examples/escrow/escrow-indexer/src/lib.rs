extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(manifest = "examples/escrow/escrow-indexer/escrow_indexer.manifest.yaml")]
mod escrow_indexer {
    fn log_something() {}
}
