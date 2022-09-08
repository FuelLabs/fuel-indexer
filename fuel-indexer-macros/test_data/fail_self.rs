use fuel_indexer_macros::indexer;


#[indexer(
    abi = "./test_data/contracts-abi.json",
    namespace = "test_namespace",
    schema = "./test_data/schema.graphql",
)]
mod indexer {
    fn function_one(self, event: SomeEvent) {
        let SomeEvent { id, account } = event;

        let t1 = Thing1 { id, account };
        t1.save();
    }
}


