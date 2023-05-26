extern crate alloc;
use fuel_indexer_macros::indexer;

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
                    id,
                }
            }
            ConsensusData::PoA(poa) => {
                let id = 1;
                Consensus {
                    unknown: None,
                    genesis: None,
                    label: ConsensusLabel::PoA.to_string(),
                    poa: Some(
                        PoA {
                            signature: poa.signature,
                        }
                        .into(),
                    ),
                    id,
                }
            }
            ConsensusData::UnknownConsensus => {
                let id = 1;
                Consensus {
                    unknown: Some(Unknown { value: true }.into()),
                    genesis: None,
                    label: ConsensusLabel::Unknown.to_string(),
                    poa: None,
                    id,
                }
            }
        }
    }
}

impl From<ClientWitness> for Witness {
    fn from(w: ClientWitness) -> Self {
        Self {
            data: w.into_inner().into(),
        }
    }
}

impl From<ClientTxPointer> for TxPointer {
    fn from(tx_pointer: ClientTxPointer) -> Self {
        let ClientTxPointer {
            block_height,
            tx_index,
        } = tx_pointer;
        Self {
            id: 1,
            block_height,
            tx_index: tx_index as u32,
        }
    }
}

impl From<ClientInputCoin> for InputCoin {
    fn from(input: ClientInputCoin) -> Self {
        let ClientInputCoin {
            #[allow(unused)]
            utxo_id,
            owner,
            amount,
            asset_id,
            tx_pointer,
            witness_index,
            maturity,
            predicate,
            predicate_data,
        } = input;

        let id = 1; // Create u64 from input parts
        let ptr = TxPointer::load(id).unwrap_or_else(|| {
            let ptr = TxPointer::from(tx_pointer);
            ptr.save();
            ptr
        });

        Self {
            id: 1,
            utxo_id: 1,
            owner,
            amount,
            asset_id,
            tx_pointer: ptr.id,
            witness_index: witness_index as i64,
            maturity: maturity as u64,
            predicate,
            predicate_data,
        }
    }
}

impl From<u64> for ContractIdFragment {
    fn from(id: u64) -> Self {
        Self { id }
    }
}

#[allow(unused)]
impl From<ClientInputContract> for InputContract {
    fn from(input: ClientInputContract) -> Self {
        let ClientInputContract {
            utxo_id,
            balance_root,
            state_root,
            tx_pointer,
            contract_id,
        } = input;

        let id = 1; // Create u64 from `contract_id`
        let contract = ContractIdFragment::load(id).unwrap_or_else(|| {
            let contract = ContractIdFragment::from(id);
            contract.save();
            contract
        });

        let id = 1; // Create u64 from input parts
        let ptr = TxPointer::load(id).unwrap_or_else(|| {
            let ptr = TxPointer::from(tx_pointer);
            ptr.save();
            ptr
        });

        Self {
            id,
            utxo_id: 1,
            balance_root,
            state_root,
            tx_pointer: ptr.id,
            contract: contract.id,
        }
    }
}

impl From<ClientInput> for Input {
    fn from(input: ClientInput) -> Self {
        match input {
            ClientInput::Coin(input) => {
                let id = 1; // Create u64 from input parts
                let coin = InputCoin::load(id).unwrap_or_else(|| {
                    let coin = InputCoin::from(input);
                    coin.save();
                    coin
                });

                let id = 1;
                let input = Input {
                    id,
                    coin: Some(coin.id),
                    contract: None,
                    message: None,
                };
                input.save();
                input
            }
            ClientInput::Contract(input) => {
                let id = 1; // Create u64 from input parts
                let contract = InputContract::load(id).unwrap_or_else(|| {
                    let contract = InputContract::from(input);
                    contract.save();
                    contract
                });

                let id = 1;
                let input = Input {
                    id,
                    coin: None,
                    contract: Some(contract.id),
                    message: None,
                };
                input.save();
                input
            }
            _ => unimplemented!(),
            // ClientInput::Message(input) => Input::Message(input.into()),
        }
    }
}

#[indexer(manifest = "examples/fuel-explorer/fuel-explorer/fuel_explorer.manifest.yaml")]
pub mod explorer_index {

    fn index_block(block_data: BlockData) {
        let id = 1; // Create u64 from block parts
        let header = Header {
            id, // Create u64 from header parts
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

        let id = 1;
        let block_frag = BlockIdFragment { id };

        block_frag.save();

        let block = Block {
            id, // Create u64 from block parts
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
                    // let outputs = t
                    //     .outputs()
                    //     .iter()
                    //     .map(|o| o.to_owned().into())
                    //     .collect::<Vec<Output>>();
                    let witnesses = t
                        .witnesses()
                        .iter()
                        .map(|w| w.to_owned().into())
                        .collect::<Vec<Witness>>();

                    let script_tx_frag = TransactionIdFragment { id: 1 };
                    script_tx_frag.save();

                    let create_tx = CreateTransaction {
                        id: 1, // Create u64 from tx parts
                        gas_limit: *gas_limit,
                        gas_price: *gas_price,
                        maturity: *maturity as u32,

                        // TODO: Where do these come from?
                        bytecode_length: 0,
                        bytecode_witness_index: 0,

                        // TODO: Pending list types
                        // storage_slots: [],
                        // inputs: [],
                        // inputs: [],
                        // outputs: [],
                        // witnesses: [],
                        salt: Salt::default(),

                        // TODO: Where do these come from?
                        metadata: Some(Json::default()),
                    };

                    create_tx.save();
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
                    let witnesses = t
                        .witnesses()
                        .iter()
                        .map(|w| w.to_owned().into())
                        .collect::<Vec<Witness>>();
                    let storage_slots = t.storage_slots();

                    // Create u64 from tx parts
                    let create_tx_frag = TransactionIdFragment { id: 1 };
                    create_tx_frag.save();
                }
                #[allow(unused)]
                ClientTransaction::Mint(t) => {
                    let tx_pointer = t.tx_pointer();
                    let outputs = t.outputs();

                    // Create u64 from tx parts
                    let mint_tx_frag = TransactionIdFragment { id: 1 };
                    mint_tx_frag.save();
                }
            }

            for receipt in transaction.receipts.iter() {
                match receipt {
                    ClientReceipt::Call { .. } => {}
                    #[allow(unused)]
                    ClientReceipt::ReturnData { .. } => {}
                    #[allow(unused)]
                    ClientReceipt::Transfer { .. } => {}
                    #[allow(unused)]
                    ClientReceipt::TransferOut { .. } => {}
                    #[allow(unused)]
                    ClientReceipt::Log { .. } => {}
                    #[allow(unused)]
                    ClientReceipt::LogData { .. } => {}
                    #[allow(unused)]
                    ClientReceipt::ScriptResult { .. } => {}
                    #[allow(unused)]
                    ClientReceipt::MessageOut { .. } => {}
                    _ => {
                        Logger::info("This Receipt type is not handled yet.");
                    }
                }
            }
        }
    }
}
