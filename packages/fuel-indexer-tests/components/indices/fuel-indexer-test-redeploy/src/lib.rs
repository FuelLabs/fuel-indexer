extern crate alloc;

use fuel_indexer_utils::prelude::*;

#[indexer(
    manifest = "packages/fuel-indexer-tests/components/indices/fuel-indexer-test-redeploy/fuel_indexer_test.yaml"
)]
mod fuel_indexer_test {
    fn fuel_indexer_test_ping(_: SomeEvent) {
        let entity = DifferentEntity {
            id: 0,
            value: 0,
            message: "hello".to_string(),
        };
        entity.save();
    }
}
