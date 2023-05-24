extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

impl From<Bytes32> for Consensus {
    fn from(bytes: Bytes32) -> Self {
        Consensus {
            id: 1,
            genesis: None,
            poa: None,
            unknown: None,
        }
    }
}

#[indexer(manifest = "examples/fuel-explorer/fuel-explorer/fuel_explorer.manifest.yaml")]
pub mod explorer_index {

    fn index_block(block: Block) {
        let header = Headers {
            id: 1,
            block_id: block.header.id.0,
            da_height: block.header.da_height,
            transactions_count: block.header.transactions_count,
            message_receipt_count: block.header.output_messages_count,
            transactions_root: block.header.transactions_root.0,
            message_receipt_root: block.header.output_messages_root.0,
            height: block.header.height,
            prev_root: block.header.prev_root.0,
            timestamp: Some(block.header.time),
            application_hash: block.header.application_hash.0,
        };

        header.save();

        let consensus = Consensus::from(block.header.application_hash);
        consensus.save();

        let block_data = Blocks {
            id: 1,
            block_id: block.header.id.0,
            header: header.id,
            consensus: consensus.id,
        };

        Logger::info("hello, john!");
    }
}
