extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    manifest = "packages/fuel-indexer-tests/components/indices/simple-wasm/manifest.yaml"
)]
pub mod test_namespace {

    fn function_one(event: SomeEvent, event2: AnotherEvent) {
        let SomeEvent { id, account } = event;
        let AnotherEvent { hash, .. } = event2;

        let t1 = Thing1 {
            id,
            account: Address::from(account.0),
        };
        t1.save();

        let t2 = Thing2 {
            id,
            account: Address::from(account.0),
            hash: Bytes32::from(hash.0),
        };

        t2.save();
    }

    fn function_two(event: AnotherEvent) {
        let AnotherEvent { id, hash, .. } = event;

        let Thing1 { account, .. } = match Thing1::load(id) {
            Some(o) => o,
            None => Thing1 {
                id,
                account: Address::from(hash.0),
            },
        };

        let t2 = Thing2 {
            id,
            account,
            hash: Bytes32::from(hash.0),
        };

        t2.save();
    }

    fn function_three(event: SomeEvent) {
        let SomeEvent { id, account } = event;
        let t1 = Thing1 {
            id,
            account: Address::from(account.0),
        };
        t1.save();
    }
}
