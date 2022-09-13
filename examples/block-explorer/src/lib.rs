extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "examples/counter/contracts/counter/out/debug/counter-abi.json",
    namespace = "block_explorer",
    identifier = "block_index1",
    schema = "schema/block.graphql"
)]
mod block_indexer {

    fn count_handler(event: CountEvent) {
        let count = Count {
            id: event.id,
            timestamp: event.timestamp,
            count: event.count,
        };

        count.save()
    }

    #[block]
    fn block_handler(block: BlockData) {
        let blk = Block { id: 1, height: 1 };
        blk.save();

        for (i, _receipts) in block.transactions.iter().enumerate() {
            let tx = Transaction {
                id: i as u64,
                block: blk.id,
            };
            tx.save();
        }
    }
}
