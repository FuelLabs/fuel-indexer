use fuel_indexer_macros::indexer;

#[indexer()]
mod indexer {
    fn function_one(event: SomeEvent) {
        let SomeEvent { id, account } = event;

        let t1 = Thing1 { id, account };
        t1.save();
    }

    fn function_one(event: SomeEvent) {
        let SomeEvent { id, account } = event;

        assert_eq!(id, 0);
        assert_eq!(account, Address::try_from([0; 32]).expect("failed"));
    }
}
