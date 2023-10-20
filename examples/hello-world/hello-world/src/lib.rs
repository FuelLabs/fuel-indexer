extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "examples/hello-world/hello-world/hello_world.manifest.yaml")]
pub mod hello_world_index_mod {

    fn hello_world_handler(block_data: BlockData) {
        let mut transactions = vec![];

        for transaction in block_data.transactions.iter() {
            let tx = Transaction::new(Bytes32::from(<[u8; 32]>::from(transaction.id)));
            tx.save();
            transactions.push(tx.id);
        }

        let block =
            Block::new(block_data.header.height.into(), block_data.id, transactions);
        block.save();
    }
}
