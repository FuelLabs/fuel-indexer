extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[no_mangle]
fn ff_log_data(_inp: ()) {}

#[no_mangle]
fn ff_put_object(_inp: ()) {}

#[no_mangle]
fn ff_put_many_to_many_record(_inp: ()) {}

#[indexer(manifest = "packages/fuel-indexer-tests/trybuild/simple_wasm.yaml")]
mod indexer {
    fn function_one() {
        let t1 = Thing1 { id: uid([1]), account: Address::default() };
        t1.save();
    }
}
