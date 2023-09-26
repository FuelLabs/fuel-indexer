extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer_test.manifest.yaml")]
pub mod indexer_test_index_mod {

    fn indexer_test_handler(block_data: BlockData) {
        if block_data.header.height % 1000 == 0 {
            info!("Processing Block#{}. (>'.')>", block_data.header.height);
        }
        
        let block = Block::new(block_data.header.height.into(), block_data.id);
        block.save();

        for transaction in block_data.transactions.iter() {
            let tx = Transaction::new(block_data.id, Bytes32::from(<[u8; 32]>::from(transaction.id)));
            tx.save();
        }
    }
}
