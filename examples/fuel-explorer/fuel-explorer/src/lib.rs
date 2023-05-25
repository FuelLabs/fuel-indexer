extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

pub enum ConsensusLabel {
    Unknown,
    Genesis,
    PoA,
}

impl ToString for ConsensusLabel {
    fn to_string(&self) -> String {
        match self {
            ConsensusLabel::Unknown => "Consensus::Unknown".to_string(),
            ConsensusLabel::Genesis => "Consensus::Genesis".to_string(),
            ConsensusLabel::PoA => "Consensus::PoA".to_string(),
        }
    }
}

// TODO: https://github.com/FuelLabs/fuel-indexer/issues/286
impl From<ConsensusData> for Consensus {
    fn from(consensus: ConsensusData) -> Self {
        match consensus {
            ConsensusData::Genesis(GenesisConsensus {
                chain_config_hash,
                coins_root,
                contracts_root,
                messages_root,
            }) => {
                let id = 1;
                let genesis = Genesis::load(id).unwrap_or(Genesis {
                    chain_config_hash,
                    coins_root,
                    contracts_root,
                    messages_root,
                    id,
                });

                Consensus {
                    unknown: None,
                    poa: None,
                    genesis: Some(genesis.id),
                    label: ConsensusLabel::Genesis.to_string(),
                    id: 1,
                }
            }
            ConsensusData::PoA(poa) => Consensus {
                unknown: None,
                genesis: None,
                label: ConsensusLabel::PoA.to_string(),
                poa: Some(
                    PoA {
                        signature: poa.signature,
                    }
                    .into(),
                ),
                id: 1,
            },
            ConsensusData::UnknownConsensus => Consensus {
                unknown: Some(Unknown { value: true }.into()),
                genesis: None,
                label: ConsensusLabel::Unknown.to_string(),
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

        let consensus = Consensus::from(block.consensus);
        consensus.save();

        let block = Block {
            id: 1,
            block_id: block.header.id,
            header: header.id,
            consensus: consensus.id,
        };

        Logger::info("hello, world!");

        block.save();
    }
}
