extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    manifest = "packages/fuel-indexer-tests/components/indices/two-contracts/two_contracts.manifest.yaml"
)]
mod two_contracts_one {

    fn test_two_contracts_one_foo(foo: Foo) {
        Logger::info("two_contracts_test_1 handling a foo event.");

        let entity = FooEntity {
            id: 123,
            num: foo.num,
        };
        entity.save();
    }
}
