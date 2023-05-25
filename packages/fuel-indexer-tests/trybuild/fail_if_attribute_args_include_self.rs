use fuel_indexer_utils::macros::indexer;

#[indexer(manifest = "packages/fuel-indexer-tests/trybuild/simple_wasm.yaml")]
mod indexer {
    fn function_one(self, event: SomeEvent) {
        let SomeEvent { id, account } = event;

        let t1 = Thing1 { id, account };
        t1.save();
    }
}
