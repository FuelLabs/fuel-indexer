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

    fn index_block(block_data: BlockData) {
        let header = Header {
            id: 1,
            block_id: block_data.header.id,
            da_height: block_data.header.da_height,
            transactions_count: block_data.header.transactions_count,
            message_receipt_count: block_data.header.output_messages_count,
            transactions_root: block_data.header.transactions_root,
            message_receipt_root: block_data.header.output_messages_root,
            height: block_data.header.height,
            prev_root: block_data.header.prev_root,
            time: block_data.header.time,
            application_hash: block_data.header.application_hash,
        };

        header.save();

        let consensus = Consensus::from(block_data.consensus);
        consensus.save();

        let block = Block {
            id: 1,
            block_id: block_data.header.id,
            header: header.id,
            consensus: consensus.id,
        };

        // Save partial block
        block.save();

        for transaction in block_data.transactions.iter() {
            match &transaction.transaction {
                #[allow(unused)]
                ClientTransaction::Script(t) => {
                    let gas_limit = t.gas_limit();
                    let gas_price = t.gas_price();
                    let maturity = t.maturity();
                    let script = t.script();
                    let script_data = t.script_data();
                    let receipts_root = t.receipts_root();
                    let inputs = t.inputs();
                    let outputs = t.outputs();
                    let witnesses = t.witnesses();
                }
                #[allow(unused)]
                ClientTransaction::Create(t) => {
                    let gas_limit = t.gas_limit();
                    let gas_price = t.gas_price();
                    let maturity = t.maturity();
                    let salt = t.salt();
                    let bytecode_length = t.bytecode_length();
                    let bytecode_witness_index = t.bytecode_witness_index();
                    let inputs = t.inputs();
                    let outputs = t.outputs();
                    let witnesses = t.witnesses();
                    let storage_slots = t.storage_slots();
                }
                #[allow(unused)]
                ClientTransaction::Mint(t) => {
                    let tx_pointer = t.tx_pointer();
                    let outputs = t.outputs();
                }
            }

            for receipt in transaction.receipts.iter() {
                match receipt {
                    Receipt::Call { .. } => {}
                    #[allow(unused)]
                    Receipt::ReturnData { .. } => {}
                    #[allow(unused)]
                    Receipt::Transfer { .. } => {}
                    #[allow(unused)]
                    Receipt::TransferOut { .. } => {}
                    #[allow(unused)]
                    Receipt::Log { .. } => {}
                    #[allow(unused)]
                    Receipt::LogData { .. } => {}
                    #[allow(unused)]
                    Receipt::ScriptResult { .. } => {}
                    #[allow(unused)]
                    Receipt::MessageOut { .. } => {}
                    _ => {
                        Logger::info("This Receipt type is not handled yet.");
                    }
                }
            }
        }
    }
}
