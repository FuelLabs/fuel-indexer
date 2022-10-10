use fuel_indexer_macros::indexer;

#[indexer(
    manifest = "fuel-indexer-tests/assets/macros/simple_wasm.yaml"
)]
mod indexer {
    fn function_one(self, event: SomeEvent) {
        let SomeEvent { id, account } = event;

        let t1 = Thing1 { id, account };
        t1.save();
    }
}
