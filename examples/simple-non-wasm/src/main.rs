use fuel_indexer_derive::indexer;

#[indexer("counter", "schema/counter.graphql", "contracts/counter/out/debug/counter-abi.json")]
pub mod indexer_namespace {
    fn count_handler(event: Count) {}

    fn another_count_handler(event: AnotherCount) {}
}

fn main() {}
