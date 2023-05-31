extern crate alloc;
use fuel_indexer_utils::prelude::*;

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

impl From<fuel::Genesis> for Genesis {
    fn from(genesis: fuel::Genesis) -> Self {
        let fuel::Genesis {
            chain_config_hash,
            coins_root,
            contracts_root,
            messages_root,
            ..
        } = genesis;

        let id = 1;
        Self {
            id,
            chain_config_hash,
            coins_root,
            contracts_root,
            messages_root,
        }
    }
}

impl From<fuel::Consensus> for Consensus {
    fn from(consensus: fuel::Consensus) -> Self {
        match consensus {
            fuel::Consensus::Genesis(g) => {
                let id = 1;
                let genesis = Genesis::load(id).unwrap_or_else(|| {
                    let g: Genesis = g.into();
                    g.save();
                    g
                });

                Consensus {
                    unknown: None,
                    genesis: Some(genesis.id),
                    label: ConsensusLabel::Genesis.to_string(),
                    poa: None,
                    id,
                }
            }
            fuel::Consensus::PoA(poa) => {
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
            fuel::Consensus::Unknown => {
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

impl From<fuel::Witness> for Witness {
    fn from(w: fuel::Witness) -> Self {
        Self {
            data: w.into_inner().into(),
        }
    }
}

impl From<fuel::TxPointer> for TxPointer {
    fn from(tx_pointer: fuel::TxPointer) -> Self {
        let fuel::TxPointer {
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

impl From<fuel::InputCoin> for InputCoin {
    fn from(input: fuel::InputCoin) -> Self {
        let fuel::InputCoin {
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
impl From<fuel::InputContract> for InputContract {
    fn from(input: fuel::InputContract) -> Self {
        let fuel::InputContract {
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

impl From<fuel::Input> for Input {
    fn from(input: fuel::Input) -> Self {
        match input {
            fuel::Input::Coin(input) => {
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
            fuel::Input::Contract(input) => {
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
            // fuel::Input::Message(input) => Input::Message(input.into()),
        }
    }
}

impl From<fuel::CoinOutput> for CoinOutput {
    fn from(output: fuel::CoinOutput) -> Self {
        let fuel::CoinOutput {
            to,
            amount,
            asset_id,
        } = output;

        let id = 1; // Create u64 from output parts
        Self {
            id,
            recipient: to,
            amount,
            asset_id,
        }
    }
}

impl From<fuel::ContractOutput> for ContractOutput {
    fn from(output: fuel::ContractOutput) -> Self {
        let fuel::ContractOutput {
            input_index,
            balance_root,
            state_root,
        } = output;

        let id = 1; // Create u64 from output parts
        Self {
            id,
            input_index: input_index as i64,
            balance_root,
            state_root,
        }
    }
}

impl From<fuel::ChangeOutput> for ChangeOutput {
    fn from(output: fuel::ChangeOutput) -> Self {
        let fuel::ChangeOutput {
            to,
            amount,
            asset_id,
        } = output;

        let id = 1; // Create u64 from output parts
        Self {
            id,
            recipient: to,
            amount,
            asset_id,
        }
    }
}

impl From<fuel::VariableOutput> for VariableOutput {
    fn from(output: fuel::VariableOutput) -> Self {
        let fuel::VariableOutput {
            to,
            amount,
            asset_id,
        } = output;

        let id = 1; // Create u64 from output parts
        Self {
            id,
            recipient: to,
            amount,
            asset_id,
        }
    }
}

impl From<fuel::Output> for Output {
    fn from(output: fuel::Output) -> Self {
        match output {
            fuel::Output::CoinOutput(output) => {
                let coin = CoinOutput::from(output);
                let id = 1; // Create u64 from output parts
                Self {
                    id,
                    coin: Some(coin.id),
                    contract: None,
                    change: None,
                    variable: None,
                    contract_created: None,
                    unknown: None,
                }
            }
            fuel::Output::ContractOutput(output) => {
                let contract = ContractOutput::from(output);
                let id = 1; // Create u64 from output parts
                Self {
                    id,
                    coin: None,
                    contract: Some(contract.id),
                    change: None,
                    variable: None,
                    contract_created: None,
                    unknown: None,
                }
            }
            fuel::Output::ChangeOutput(output) => {
                let change = ChangeOutput::from(output);
                let id = 1; // Create u64 from output parts
                Self {
                    id,
                    coin: None,
                    contract: None,
                    change: Some(change.id),
                    variable: None,
                    contract_created: None,
                    unknown: None,
                }
            }
            fuel::Output::VariableOutput(output) => {
                let var = VariableOutput::from(output);
                let id = 1; // Create u64 from output parts
                Self {
                    id,
                    coin: None,
                    contract: None,
                    change: None,
                    variable: Some(var.id),
                    contract_created: None,
                    unknown: None,
                }
            }
            fuel::Output::ContractCreated(output) => {
                let contract = ContractCreated::from(output);
                let id = 1; // Create u64 from output parts
                Self {
                    id,
                    coin: None,
                    contract: None,
                    change: None,
                    variable: None,
                    contract_created: Some(contract.id),
                    unknown: None,
                }
            }
            _ => {
                Logger::warn("Unrecognized output type.");
                Self {
                    id: 1,
                    coin: None,
                    contract: None,
                    change: None,
                    variable: None,
                    contract_created: None,
                    unknown: Some(Unknown { value: true }.into()),
                }
            }
        }
    }
}

impl From<fuel::ContractCreated> for ContractCreated {
    fn from(output: fuel::ContractCreated) -> Self {
        let fuel::ContractCreated {
            #[allow(unused)]
            contract,
            state_root,
        } = output;

        let id = 1; // Create u64 from contract ID
        let contract = Contract::load(id).unwrap();

        let id = 1; // Create u64 from output parts
        Self {
            id,
            contract: contract.id,
            state_root,
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
        let _foo = "bar";
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
                fuel::Transaction::Script(t) => {
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
                fuel::Transaction::Create(t) => {
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
                fuel::Transaction::Mint(t) => {
                    let tx_pointer = t.tx_pointer();
                    let outputs = t.outputs();

                    // Create u64 from tx parts
                    let mint_tx_frag = TransactionIdFragment { id: 1 };
                    mint_tx_frag.save();
                }
            }

            for receipt in transaction.receipts.iter() {
                match receipt {
                    fuel::Receipt::Call { .. } => {}
                    #[allow(unused)]
                    fuel::Receipt::ReturnData { .. } => {}
                    #[allow(unused)]
                    fuel::Receipt::Transfer { .. } => {}
                    #[allow(unused)]
                    fuel::Receipt::TransferOut { .. } => {}
                    #[allow(unused)]
                    fuel::Receipt::Log { .. } => {}
                    #[allow(unused)]
                    fuel::Receipt::LogData { .. } => {}
                    #[allow(unused)]
                    fuel::Receipt::ScriptResult { .. } => {}
                    #[allow(unused)]
                    fuel::Receipt::MessageOut { .. } => {}
                    _ => {
                        Logger::info("This Receipt type is not handled yet.");
                    }
                }
            }
        }
    }
}
