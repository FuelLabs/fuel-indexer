extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;
use fuels::types::param_types::ParamType;

#[indexer(manifest = "examples/escrow/escrow-indexer/escrow_indexer.manifest.yaml")]
mod escrow_indexer {
    fn index_accepted_arbiter_event(event: AcceptedArbiterEvent, block_data: BlockData) {
        let identifier = event.identifier;

        let block = Block {
            id: first8_bytes_to_u64(block_data.id),
            height: block_data.height,
            timestamp: block_data.time,
        };

        block.save();
    }
}
