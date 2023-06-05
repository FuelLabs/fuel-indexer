extern crate alloc;
use fuel_indexer_utils::prelude::*;

impl From<fuel::ReturnType> for ReturnType {
    fn from(value: fuel::ReturnType) -> Self {
        match value {
            fuel::ReturnType::Return => ReturnType::Return,
            fuel::ReturnType::Revert => ReturnType::Revert,
            fuel::ReturnType::ReturnData => ReturnType::ReturnData,
        }
    }
}

impl From<fuel::ProgramState> for ProgramState {
    fn from(state: fuel::ProgramState) -> Self {
        let fuel::ProgramState { return_type, data } = state;
        Self {
            return_type: ReturnType::from(return_type).into(),
            data,
        }
    }
}

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

        // TODO: Create UID here.
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
                // TODO: Create UID here.
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
                // TODO: Create UID here.
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
                // TODO: Create UID here.
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
        // TODO: Create UID here.
        let id = 1;
        Self {
            id,
            block_height,
            tx_index,
        }
    }
}

impl From<fuel::UtxoId> for UtxoId {
    fn from(utxo_id: fuel::UtxoId) -> Self {
        // TODO: Create UID here.
        let id = 1;
        Self {
            id,
            tx_id: *utxo_id.tx_id(),
            output_index: utxo_id.output_index().into(),
        }
    }
}

impl From<fuel::InputCoin> for InputCoin {
    fn from(input: fuel::InputCoin) -> Self {
        let fuel::InputCoin {
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

        // TODO: Create UID here.
        // let id = utxo_id.tx_id();
        let id = 1;
        let utxo = UtxoId::load(id).unwrap_or_else(|| {
            let utxo = UtxoId::from(utxo_id);
            utxo.save();
            utxo
        });

        // TODO: Create UID here.
        let id = 1;
        let ptr = TxPointer::load(id).unwrap_or_else(|| {
            let ptr = TxPointer::from(tx_pointer);
            ptr.save();
            ptr
        });

        // TODO: Create UID here.
        let id = 1;
        Self {
            id,
            utxo_id: utxo.id,
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

        // TODO: Create UID here.
        let id = 1;
        let contract = ContractIdFragment::load(id).unwrap_or_else(|| {
            let contract = ContractIdFragment::from(id);
            contract.save();
            contract
        });

        // TODO: Create UID here.
        let id = 1;
        let ptr = TxPointer::load(id).unwrap_or_else(|| {
            let ptr = TxPointer::from(tx_pointer);
            ptr.save();
            ptr
        });

        // TODO: Create UID here.
        // let id = utxo_id.tx_id();
        let id = 1;
        let utxo = UtxoId::load(id).unwrap_or_else(|| {
            let utxo = UtxoId::from(utxo_id);
            utxo.save();
            utxo
        });

        Self {
            id,
            utxo_id: utxo.id,
            balance_root,
            state_root,
            tx_pointer: ptr.id,
            contract: contract.id,
        }
    }
}

impl From<fuel::InputMessage> for InputMessage {
    fn from(input: fuel::InputMessage) -> Self {
        let fuel::InputMessage {
            sender,
            recipient,
            amount,
            nonce,
            witness_index,
            data,
            predicate,
            predicate_data,
        } = input;

        // TODO: Create UID here.
        let id = 1;

        Self {
            id,
            sender,
            recipient,
            amount,
            nonce,
            witness_index: witness_index.into(),
            data,
            predicate,
            predicate_data,
        }
    }
}

impl From<fuel::Input> for Input {
    fn from(input: fuel::Input) -> Self {
        match input {
            fuel::Input::Coin(input) => {
                // TODO: Create UID here.
                let id = 1;
                let coin = InputCoin::load(id).unwrap_or_else(|| {
                    let coin = InputCoin::from(input);
                    coin.save();
                    coin
                });

                // TODO: Create UID here.
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
                // TODO: Create UID here.
                let id = 1;
                let contract = InputContract::load(id).unwrap_or_else(|| {
                    let contract = InputContract::from(input);
                    contract.save();
                    contract
                });

                // TODO: Create UID here.
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
            fuel::Input::Message(input) => {
                // TODO: Create UID here.
                let id = 1;
                let message = InputMessage::load(id).unwrap_or_else(|| {
                    let message = InputMessage::from(input);
                    message.save();
                    message
                });

                // TODO: Create UID here.
                let id = 1;
                let input = Input {
                    id,
                    coin: None,
                    contract: None,
                    message: Some(message.id),
                };
                input.save();
                input
            }
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

        // TODO: Create UID here.
        let id = 1;
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

        // TODO: Create UID here.
        let id = 1;
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

        // TODO: Create UID here.
        let id = 1;
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

        // TODO: Create UID here.
        let id = 1;
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
                // TODO: Create UID here.
                let id = 1;
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
                // TODO: Create UID here.
                let id = 1;
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
                // TODO: Create UID here.
                let id = 1;
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
                // TODO: Create UID here.
                let id = 1;
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
                // TODO: Create UID here.
                let id = 1;
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
                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
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
        #[allow(unused)]
        let fuel::ContractCreated {
            contract_id,
            state_root,
        } = output;

        // TODO: Create UID here.
        let id = 1;
        let contract = Contract::load(id).unwrap();

        // TODO: Create UID here.
        let id = 1;
        Self {
            id,
            contract: contract.id,
            state_root,
        }
    }
}

impl From<fuel::TransactionStatus> for TransactionStatus {
    fn from(status: fuel::TransactionStatus) -> Self {
        match status {
            #[allow(unused)]
            fuel::TransactionStatus::Failure {
                block,
                time,
                reason,
                program_state,
            } => {
                // TODO: Create UID here.
                let id = 1;
                // TODO: Create UID here.
                let block_id = 1;
                let block = BlockIdFragment::load(block_id).unwrap();
                let program_state = program_state.map(|p| p.into());
                let failure = FailureStatus::load(id).unwrap_or_else(|| {
                    let failure = FailureStatus {
                        id,
                        block: block.id,
                        time,
                        reason: reason.into(),
                        program_state,
                    };

                    failure.save();
                    failure
                });

                Self {
                    id,
                    submitted_status: None,
                    squeezed_out_status: None,
                    failure_status: Some(failure.id),
                    success_status: None,
                    unknown_status: None,
                }
            }
            fuel::TransactionStatus::SqueezedOut { reason } => {
                // TODO: Create UID here.
                let id = 1;
                let squeezed_out = SqueezedOutStatus::load(id).unwrap_or_else(|| {
                    let squeezed_out = SqueezedOutStatus {
                        id,
                        reason: reason.into(),
                    };

                    squeezed_out.save();
                    squeezed_out
                });

                Self {
                    id,
                    submitted_status: None,
                    squeezed_out_status: Some(squeezed_out.id),
                    failure_status: None,
                    success_status: None,
                    unknown_status: None,
                }
            }
            #[allow(unused)]
            fuel::TransactionStatus::Success {
                block,
                time,
                program_state,
            } => {
                // TODO: Create UID here.
                let id = 1;
                // TODO: Create UID here.
                let block_id = 1;
                let block = BlockIdFragment::load(block_id).unwrap();
                let program_state = program_state.map(|p| p.into());
                let success = SuccessStatus::load(id).unwrap_or_else(|| {
                    let success = SuccessStatus {
                        id,
                        block: block.id,
                        time,
                        program_state,
                    };

                    success.save();
                    success
                });

                Self {
                    id,
                    submitted_status: None,
                    squeezed_out_status: None,
                    failure_status: None,
                    success_status: Some(success.id),
                    unknown_status: None,
                }
            }
            fuel::TransactionStatus::Submitted { submitted_at } => {
                // TODO: Create UID here.
                let id = 1;
                let submitted = SubmittedStatus::load(id).unwrap_or_else(|| {
                    let submitted = SubmittedStatus {
                        id,
                        time: submitted_at,
                    };

                    submitted.save();
                    submitted
                });

                Self {
                    id,
                    submitted_status: Some(submitted.id),
                    squeezed_out_status: None,
                    failure_status: None,
                    success_status: None,
                    unknown_status: None,
                }
            }
        }
    }
}

#[indexer(manifest = "examples/fuel-explorer/fuel-explorer/fuel_explorer.manifest.yaml")]
pub mod explorer_index {

    fn index_block(block_data: BlockData) {
        // TODO: Create UID here.
        let id = 1;
        let header = Header {
            id,
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

        // TODO: Create UID here.
        let id = 1;
        let block_frag = BlockIdFragment {
            id,
            hash: Bytes32::default(),
        };

        block_frag.save();

        // TODO: Create UID here.
        let id = 1;
        let block = Block {
            id,
            block_id: block_data.header.id,
            header: header.id,
            consensus: consensus.id,
        };

        // Save partial block
        block.save();

        for transaction in block_data.transactions.iter() {
            let _tx_status = &transaction.status;

            match &transaction.transaction {
                #[allow(unused)]
                fuel::Transaction::Script(fuel::Script {
                    gas_limit,
                    gas_price,
                    maturity,
                    script,
                    script_data,
                    receipts_root,
                    inputs,
                    outputs,
                    witnesses,
                    metadata,
                }) => {
                    let outputs = outputs
                        .iter()
                        .map(|o| Output::from(o.to_owned()))
                        .collect::<Vec<Output>>();
                    let witnesses = witnesses
                        .iter()
                        .map(|w| w.to_owned().into())
                        .collect::<Vec<Witness>>();

                    // TODO: Create UID here.
                    let id = 1;
                    let script_tx = ScriptTransaction {
                        id,
                        gas_limit: *gas_limit,
                        gas_price: *gas_price,
                        maturity: *maturity as u32,
                        script: script.to_owned().into(),
                        // storage_slots: [],
                        // inputs: [],
                        // inputs: [],
                        // outputs: [],
                        // witnesses: [],
                        receipts_root: *receipts_root,
                        metadata: metadata.to_owned().map(|m| m.into()),
                    };

                    let script_tx_frag = TransactionIdFragment { id };
                    script_tx_frag.save();

                    script_tx.save();
                }
                #[allow(unused)]
                fuel::Transaction::Create(fuel::Create {
                    gas_limit,
                    gas_price,
                    maturity,
                    bytecode_length,
                    bytecode_witness_index,
                    inputs,
                    outputs,
                    witnesses,
                    salt,
                    storage_slots,
                    metadata,
                }) => {
                    let outputs = outputs
                        .iter()
                        .map(|o| Output::from(o.to_owned()))
                        .collect::<Vec<Output>>();
                    let witnesses = witnesses
                        .iter()
                        .map(|w| w.to_owned().into())
                        .collect::<Vec<Witness>>();

                    // TODO: Create UID here.
                    let id = 1;
                    let create_tx = CreateTransaction {
                        id,
                        gas_limit: *gas_limit,
                        gas_price: *gas_price,
                        maturity: *maturity as u32,
                        bytecode_length: *bytecode_length,
                        bytecode_witness_index: *bytecode_witness_index,
                        // storage_slots: [],
                        // inputs: [],
                        // inputs: [],
                        // outputs: [],
                        // witnesses: [],
                        salt: *salt,
                        metadata: metadata.to_owned().map(|m| m.into()),
                    };

                    let create_tx_frag = TransactionIdFragment { id };
                    create_tx_frag.save();

                    create_tx.save();
                }
                #[allow(unused)]
                fuel::Transaction::Mint(fuel::Mint {
                    tx_pointer,
                    outputs,
                    metadata,
                }) => {
                    // TODO: Create UID here.
                    let mint_tx_frag = TransactionIdFragment { id: 1 };
                    mint_tx_frag.save();
                }
            }

            for receipt in transaction.receipts.iter() {
                // TODO: Capture all contract IDs from all receipts
                // TODO: Capture all addresses from all receipts
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
