extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "examples/native-websockets/manifest.yaml")]
pub mod native_websockets {

    fn foobar(event: SomeEvent, event2: AnotherEvent) {
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
}
