extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "tests/contracts/counter/out/debug/counter-abi.json",
    namespace = "counter",
    identifier = "index1",
    schema = "./../../../assets/schema/counter.graphql"
)]
mod simple_native {
    fn count_handler(event: CountEvent) {
        let count = Count {
            id: event.id,
            timestamp: event.timestamp,
            count: event.count,
        };

        count.save()
    }
}
