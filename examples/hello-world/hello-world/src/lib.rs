extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "examples/hello-world/hello-world/hello_world.manifest.yaml")]
pub mod hello_world_index_mod {

    fn hello_world_handler(block_data: BlockData) {
        let block = Block::new(block_data.header.height.into(), block_data.id);
        block.save();

        for transaction in block_data.transactions.iter() {
            let tx = Transaction::new(
                block_data.id,
                Bytes32::from(<[u8; 32]>::from(transaction.id)),
            );
            tx.save();
        }
    }
}
