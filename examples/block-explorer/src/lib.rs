extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "examples/counter/contracts/counter/out/debug/counter-abi.json",
    namespace = "block_explorer",
    identifier = "block_index1",
    schema = "../../fuel-indexer-macros/src/explorer.graphql"
)]
mod block_indexer {

    #[block]
    fn block_handler(block: BlockData) {
        // Using the Count entity from the GraphQL schema
    }
}
