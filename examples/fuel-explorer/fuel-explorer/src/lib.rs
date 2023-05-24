extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

// TODO: https://github.com/FuelLabs/fuel-indexer/issues/286
impl From<Consensus> for BlockConensus {
    fn from(consensus: Consensus) -> Self {
        match consensus {
            Consensus::Genesis(Genesis {
                chain_config_hash,
                coins_root,
                contracts_root,
                messages_root,
            }) => {
                let id = 1;
                let genesis = GenesisConsensus::load(id).unwrap_or(GenesisConsensus {
                    chain_config_hash,
                    coins_root,
                    contracts_root,
                    messages_root,
                    id,
                });

                BlockConensus {
                    unknown: None,
                    poa: None,
                    genesis: Some(genesis.id),
                    id: 1,
                }
            }
            Consensus::PoA(poa) => BlockConensus {
                unknown: None,
                genesis: None,
                poa: Some(
                    PoAConsensus {
                        signature: poa.signature,
                    }
                    .into(),
                ),
                id: 1,
            },
            Consensus::Unknown => BlockConensus {
                unknown: Some(UnknownConsensus { value: true }.into()),
                genesis: None,
                poa: None,
                id: 1,
            },
        }
    }
}

#[indexer(manifest = "examples/fuel-explorer/fuel-explorer/fuel_explorer.manifest.yaml")]
pub mod explorer_index {

    fn index_block(block: BlockData) {
        let header = Header {
            id: 1,
            block_id: block.header.id,
            da_height: block.header.da_height,
            transactions_count: block.header.transactions_count,
            message_receipt_count: block.header.output_messages_count,
            transactions_root: block.header.transactions_root,
            message_receipt_root: block.header.output_messages_root,
            height: block.header.height,
            prev_root: block.header.prev_root,
            timestamp: Some(block.header.time),
            application_hash: Some(block.header.application_hash),
        };

        header.save();

        let consensus = BlockConensus::from(block.consensus);
        consensus.save();

        let block = Block {
            id: 1,
            block_id: block.header.id,
            header: header.id,
            consensus: consensus.id,
        };

        Logger::info("hello, john!");

        block.save();
    }
}
