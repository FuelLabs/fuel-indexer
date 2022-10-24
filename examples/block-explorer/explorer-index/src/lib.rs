extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "examples/block-explorer/explorer_index.manifest.yaml")]
mod explorer_index {
    fn index_explorer_data(block: BlockData) {
        let block_entity = Block {
            id: block.height,
            height: block.height,
            timestamp: block.time,
            miner: block.producer,
        };

        block_entity.save();

        // let mut contracts = Vec::new();

        for _tx in block.transactions {
            continue;
            // let tx_entity = Transaction {
            //     id: i as u64 + block_entity.height,
            //     block: block_entity.id,
            //     timestamp: block_entity.timestamp,
            // };

            // for receipt in tx {
            // match receipt {
            //     Receipt::Call { id, .. } => {
            //         contracts.push(Contract { creator: *id });
            //     }
            //     Receipt::ReturnData { id, .. } => {
            //         contracts.push(Contract { creator: *id });
            //     }
            //     #[allow(unused)]
            //     Receipt::Transfer { id, to, .. } => {
            //         contracts.push(Contract { creator: *id });
            //     }
            //     #[allow(unused)]
            //     Receipt::TransferOut {
            //         id,
            //         to,
            //         amount,
            //         asset_id,
            //         ..
            //     } => {
            //         contracts.push(Contract { creator: *id });
            //     }
            //     Receipt::Log { id, .. } => {
            //         contracts.push(Contract { creator: *id });
            //     }
            //     Receipt::LogData { id, .. } => {
            //         contracts.push(Contract { creator: *id });
            //     }
            //     #[allow(unused)]
            //     Receipt::ScriptResult { result, gas_used } => {}
            //     #[allow(unused)]
            //     Receipt::MessageOut {
            //         sender,
            //         recipient,
            //         amount,
            //         ..
            //     } => {}
            //     _ => {
            //         Logger::info("This type is not handled yet. (>'.')>");
            //     }
            //     // }
            // }

            // tx_entity.save();
        }
    }
}
