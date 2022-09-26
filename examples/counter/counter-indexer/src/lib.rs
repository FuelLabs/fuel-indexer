extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "examples/counter/contracts/counter/out/debug/counter-abi.json",
    namespace = "counter",
    identifier = "index1",
    schema = "../schema/counter.graphql"
)]
mod counter {
    fn count_handler(event: CountEvent) {
        let count = Count {
            id: event.id,
            timestamp: event.timestamp,
            count: event.count,
        };

        count.save()
    }
}
