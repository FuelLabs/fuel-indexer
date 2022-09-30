use fuel_indexer_macros::indexer;

#[indexer(
    abi = "./../examples/simple-wasm/contracts/out/debug/contracts-abi.json",
    namespace = "test_namespace",
    identifier = "bar",
    schema = "./../examples/simple-wasm/schema/schema.graphql"
)]
mod indexer {
    fn function_one(self, event: SomeEvent) {
        let SomeEvent { id, account } = event;

        let t1 = Thing1 { id, account };
        t1.save();
    }
}
