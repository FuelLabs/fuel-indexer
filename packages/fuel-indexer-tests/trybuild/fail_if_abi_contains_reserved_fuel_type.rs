extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "packages/fuel-indexer-tests/trybuild/invalid_abi_type_simple_wasm.yaml")]
mod indexer {
    fn function_one(event: BlockHeight) {
        let BlockHeight { id, account } = event;

        let t1 = Thing1 { id, account };
        t1.save();
    }
}
