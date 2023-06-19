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

impl From<fuel::ClientPanicReason> for PanicReason {
    fn from(reason: fuel::ClientPanicReason) -> Self {
        match reason {
            fuel::ClientPanicReason::Success => PanicReason::Success,
            fuel::ClientPanicReason::Revert => PanicReason::Revert,
            fuel::ClientPanicReason::OutOfGas => PanicReason::OutOfGas,
            fuel::ClientPanicReason::TransactionValidity => {
                PanicReason::TransactionValidity
            }
            fuel::ClientPanicReason::MemoryOverflow => PanicReason::MemoryOverflow,
            fuel::ClientPanicReason::ArithmeticOverflow => {
                PanicReason::ArithmeticOverflow
            }
            fuel::ClientPanicReason::ContractNotFound => PanicReason::ContractNotFound,
            fuel::ClientPanicReason::MemoryOwnership => PanicReason::MemoryOwnership,
            fuel::ClientPanicReason::NotEnoughBalance => PanicReason::NotEnoughBalance,
            fuel::ClientPanicReason::ExpectedInternalContext => {
                PanicReason::ExpectedInternalContext
            }
            fuel::ClientPanicReason::AssetIdNotFound => PanicReason::AssetIdNotFound,
            fuel::ClientPanicReason::InputNotFound => PanicReason::InputNotFound,
            fuel::ClientPanicReason::OutputNotFound => PanicReason::OutputNotFound,
            fuel::ClientPanicReason::WitnessNotFound => PanicReason::WitnessNotFound,
            fuel::ClientPanicReason::TransactionMaturity => {
                PanicReason::TransactionMaturity
            }
            fuel::ClientPanicReason::InvalidMetadataIdentifier => {
                PanicReason::InvalidMetadataIdentifier
            }
            fuel::ClientPanicReason::MalformedCallStructure => {
                PanicReason::MalformedCallStructure
            }
            fuel::ClientPanicReason::ReservedRegisterNotWritable => {
                PanicReason::ReservedRegisterNotWritable
            }
            fuel::ClientPanicReason::ErrorFlag => PanicReason::ErrorFlag,
            fuel::ClientPanicReason::InvalidImmediateValue => {
                PanicReason::InvalidImmediateValue
            }
            fuel::ClientPanicReason::ExpectedCoinInput => PanicReason::ExpectedCoinInput,
            fuel::ClientPanicReason::MaxMemoryAccess => PanicReason::MaxMemoryAccess,
            fuel::ClientPanicReason::MemoryWriteOverlap => {
                PanicReason::MemoryWriteOverlap
            }
            fuel::ClientPanicReason::ContractNotInInputs => {
                PanicReason::ContractNotInInputs
            }
            fuel::ClientPanicReason::InternalBalanceOverflow => {
                PanicReason::InternalBalanceOverflow
            }
            fuel::ClientPanicReason::ContractMaxSize => PanicReason::ContractMaxSize,
            fuel::ClientPanicReason::ExpectedUnallocatedStack => {
                PanicReason::ExpectedUnallocatedStack
            }
            fuel::ClientPanicReason::MaxStaticContractsReached => {
                PanicReason::MaxStaticContractsReached
            }
            fuel::ClientPanicReason::TransferAmountCannotBeZero => {
                PanicReason::TransferAmountCannotBeZero
            }
            fuel::ClientPanicReason::ExpectedOutputVariable => {
                PanicReason::ExpectedOutputVariable
            }
            fuel::ClientPanicReason::ExpectedParentInternalContext => {
                PanicReason::ExpectedParentInternalContext
            }
            fuel::ClientPanicReason::IllegalJump => PanicReason::IllegalJump,
            fuel::ClientPanicReason::ContractIdAlreadyDeployed => {
                PanicReason::ContractIdAlreadyDeployed
            }
            _ => PanicReason::Unknown,
        }
    }
}

impl From<fuel::Consensus> for Consensus {
    fn from(consensus: fuel::Consensus) -> Self {
        match consensus {
            fuel::Consensus::Genesis(g) => Consensus::from(Genesis::new(
                g.chain_config_hash,
                g.coins_root,
                g.contracts_root,
                g.messages_root,
                ConsensusLabel::Genesis.into(),
            )),
            fuel::Consensus::PoA(poa) => {
                Consensus::from(PoA::new(poa.signature, ConsensusLabel::PoA.into()))
            }
            fuel::Consensus::Unknown => {
                Consensus::from(Unknown::new(ConsensusLabel::Unknown.into()))
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

impl From<u64> for ContractIdFragment {
    fn from(id: u64) -> Self {
        Self { id }
    }
}

impl From<fuel::Input> for Input {
    fn from(input: fuel::Input) -> Self {
        match input {
            fuel::Input::Coin(input) => {
                // TODO: Create UID here.
                let id = 1;
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

                let utxo_id = UtxoId::from(utxo_id);
                let tx_pointer = TxPointer::from(tx_pointer);

                Self {
                    id,
                    utxo_id: Some(utxo_id.id),
                    owner: Some(owner),
                    amount: Some(amount),
                    asset_id: Some(asset_id),
                    tx_pointer: Some(tx_pointer.id),
                    witness_index: Some(witness_index.into()),
                    maturity: Some(maturity),
                    predicate: Some(predicate),
                    predicate_data: Some(predicate_data.into()),
                    label: Some(InputLabel::Coin.into()),
                    balance_root: None,
                    state_root: None,
                    contract: None,
                    sender: None,
                    recipient: None,
                    nonce: None,
                    data: None,
                    is_coin: Some(true),
                    is_message: None,
                    is_contract: None,
                }
            }
            fuel::Input::Contract(input) => {
                // TODO: Create UID here.
                let id = 1;

                #[allow(unused)]
                let fuel::InputContract {
                    utxo_id,
                    balance_root,
                    state_root,
                    tx_pointer,
                    contract_id,
                } = input;

                let tx_pointer = TxPointer::from(tx_pointer);
                // TODO: derive contract ID u64 from contract_id
                let contract_id = 1;

                Self {
                    id,
                    utxo_id: None,
                    owner: None,
                    amount: None,
                    asset_id: None,
                    tx_pointer: Some(tx_pointer.id),
                    witness_index: None,
                    maturity: None,
                    predicate: None,
                    predicate_data: None,
                    label: Some(InputLabel::Contract.into()),
                    balance_root: Some(balance_root),
                    state_root: None,
                    contract: Some(contract_id),
                    sender: None,
                    recipient: None,
                    nonce: None,
                    data: None,
                    is_coin: None,
                    is_message: None,
                    is_contract: Some(true),
                }
            }
            fuel::Input::Message(input) => {
                // TODO: Create UID here.
                let id = 1;
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

                Self {
                    id,
                    utxo_id: None,
                    owner: None,
                    amount: Some(amount),
                    asset_id: None,
                    tx_pointer: None,
                    witness_index: Some(witness_index.into()),
                    maturity: None,
                    predicate: Some(predicate.into()),
                    predicate_data: Some(predicate_data.into()),
                    label: Some(InputLabel::Message.into()),
                    balance_root: None,
                    state_root: None,
                    contract: None,
                    sender: Some(sender),
                    recipient: Some(recipient),
                    nonce: Some(nonce),
                    data: Some(data.into()),
                    is_coin: None,
                    is_message: Some(true),
                    is_contract: None,
                }
            }
        }
    }
}

impl From<fuel::Output> for Output {
    fn from(output: fuel::Output) -> Self {
        match output {
            fuel::Output::CoinOutput(output) => {
                let fuel::CoinOutput {
                    to,
                    amount,
                    asset_id,
                } = output;
                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
                    recipient: Some(to),
                    amount: Some(amount),
                    asset_id: Some(asset_id),
                    contract: None,
                    input_index: None,
                    balance_root: None,
                    state_root: None,
                    is_variable: None,
                    is_contract: None,
                    is_contract_created: None,
                    is_change: None,
                    is_coin: Some(true),
                    is_unknown: None,
                    label: Some(OutputLabel::Coin.into()),
                }
            }
            fuel::Output::ContractOutput(output) => {
                let fuel::ContractOutput {
                    input_index,
                    balance_root,
                    state_root,
                } = output;
                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
                    recipient: None,
                    amount: None,
                    asset_id: None,
                    contract: None,
                    input_index: Some(input_index.into()),
                    balance_root: Some(balance_root),
                    state_root: Some(state_root),
                    is_variable: None,
                    is_contract: Some(true),
                    is_contract_created: None,
                    is_change: None,
                    is_coin: None,
                    is_unknown: None,
                    label: Some(OutputLabel::Contract.into()),
                }
            }
            fuel::Output::ChangeOutput(output) => {
                let fuel::ChangeOutput {
                    to,
                    amount,
                    asset_id,
                } = output;
                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
                    recipient: Some(to),
                    amount: Some(amount),
                    asset_id: Some(asset_id),
                    contract: None,
                    input_index: None,
                    balance_root: None,
                    state_root: None,
                    is_variable: None,
                    is_contract: None,
                    is_contract_created: None,
                    is_change: Some(true),
                    is_coin: None,
                    is_unknown: None,
                    label: Some(OutputLabel::Change.into()),
                }
            }
            fuel::Output::VariableOutput(output) => {
                let fuel::VariableOutput {
                    to,
                    amount,
                    asset_id,
                } = output;

                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
                    recipient: Some(to),
                    amount: Some(amount),
                    asset_id: Some(asset_id),
                    contract: None,
                    input_index: None,
                    balance_root: None,
                    state_root: None,
                    is_variable: Some(true),
                    is_contract: None,
                    is_contract_created: None,
                    is_change: None,
                    is_coin: None,
                    is_unknown: None,
                    label: Some(OutputLabel::Variable.into()),
                }
            }
            fuel::Output::ContractCreated(output) => {
                #[allow(unused)]
                let fuel::ContractCreated {
                    contract_id,
                    state_root,
                } = output;

                // TODO: Create UID here.
                let id = 1;

                // TODO: calculate contract ID
                let contract_id = 1;

                Self {
                    id,
                    recipient: None,
                    amount: None,
                    asset_id: None,
                    contract: Some(contract_id),
                    input_index: None,
                    balance_root: None,
                    state_root: Some(state_root),
                    is_variable: None,
                    is_contract: None,
                    is_contract_created: None,
                    is_change: None,
                    is_coin: None,
                    is_unknown: Some(true),
                    label: Some(OutputLabel::ContractCreated.into()),
                }
            }
            fuel::Output::Unknown => {
                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
                    recipient: None,
                    amount: None,
                    asset_id: None,
                    contract: None,
                    input_index: None,
                    balance_root: None,
                    state_root: None,
                    is_variable: None,
                    is_contract: None,
                    is_contract_created: None,
                    is_unknown: Some(true),
                    is_change: None,
                    is_coin: None,
                    label: Some(OutputLabel::Unknown.into()),
                }
            }
        }
    }
}

impl From<fuel::TransactionStatus> for TransactionStatus {
    fn from(status: fuel::TransactionStatus) -> Self {
        match status {
            fuel::TransactionStatus::Submitted { submitted_at } => {
                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
                    label: Some(TransactionStatusLabel::Submitted.into()),
                    time: Some(submitted_at),
                    reason: None,
                    block: None,
                    program_state: None,
                    is_submitted: Some(true),
                    is_squeezed_out: None,
                    is_failure: None,
                    is_success: None,
                    is_unknown: None,
                }
            }
            fuel::TransactionStatus::SqueezedOut { reason } => {
                // TODO: Create UID here.
                let id = 1;
                Self {
                    id,
                    label: Some(TransactionStatusLabel::SqueezedOut.into()),
                    time: None,
                    reason: Some(reason),
                    block: None,
                    program_state: None,
                    is_submitted: None,
                    is_squeezed_out: Some(true),
                    is_failure: None,
                    is_success: None,
                    is_unknown: None,
                }
            }
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
                let program_state = program_state.map(|p| p.into());

                Self {
                    id,
                    label: Some(TransactionStatusLabel::Failure.into()),
                    time: Some(time),
                    reason: None,
                    block: Some(block_id),
                    program_state,
                    is_submitted: None,
                    is_squeezed_out: None,
                    is_failure: Some(true),
                    is_success: None,
                    is_unknown: None,
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
                let program_state = program_state.map(|p| p.into());
                Self {
                    id,
                    label: Some(TransactionStatusLabel::Success.into()),
                    time: Some(time),
                    reason: None,
                    block: Some(block_id),
                    program_state,
                    is_submitted: None,
                    is_squeezed_out: None,
                    is_failure: None,
                    is_success: Some(true),
                    is_unknown: None,
                }
            }
        }
    }
}

impl From<fuel::ScriptExecutionResult> for ScriptExecutionResult {
    fn from(result: fuel::ScriptExecutionResult) -> Self {
        match result {
            fuel::ScriptExecutionResult::Success => ScriptExecutionResult::Success,
            fuel::ScriptExecutionResult::Panic => ScriptExecutionResult::Panic,
            fuel::ScriptExecutionResult::Revert => ScriptExecutionResult::Revert,
            fuel::ScriptExecutionResult::GenericFailure(_) => {
                ScriptExecutionResult::GenericFailure
            }
        }
    }
}

impl From<fuel::Receipt> for Receipt {
    fn from(receipt: fuel::Receipt) -> Self {
        match receipt {
            fuel::Receipt::Call {
                id: contract_id,
                to: recipient,
                amount,
                asset_id,
                gas,
                param1,
                param2,
                pc,
                is: isr,
            } => Self {
                amount: Some(amount),
                asset_id: Some(asset_id),
                contract_id: Some(contract_id),
                data: None,
                digest: None,
                gas_used: None,
                gas: Some(gas),
                isr: Some(isr),
                len: None,
                nonce: None,
                param1: Some(param1),
                param2: Some(param2),
                pc: Some(pc),
                ptr: None,
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: Some(fuel::Identity::ContractId(recipient)),
                result: None,
                sender: None,
                val: None,
            },
            fuel::Receipt::Return {
                id: contract_id,
                val,
                pc,
                is: isr,
            } => Self {
                amount: None,
                asset_id: None,
                contract_id: Some(contract_id),
                data: None,
                digest: None,
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: None,
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: None,
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: None,
                result: None,
                sender: None,
                val: Some(val),
            },
            fuel::Receipt::ReturnData {
                id: contract_id,
                ptr,
                len,
                digest,
                data,
                pc,
                is: isr,
            } => Self {
                amount: None,
                asset_id: None,
                contract_id: Some(contract_id),
                data: Some(data.into()),
                digest: Some(digest),
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: Some(len),
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: Some(ptr),
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: None,
                result: None,
                sender: None,
                val: None,
            },
            #[allow(unused)]
            fuel::Receipt::Panic {
                id,
                contract_id,
                reason,
                pc,
                is: isr,
            } => Self {
                amount: None,
                asset_id: None,
                contract_id: Some(id),
                data: None,
                digest: None,
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: None,
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: None,
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: Some(
                    InstructionResult {
                        reason: PanicReason::from(reason.reason().to_owned()).into(),
                        instruction: *reason.instruction(),
                    }
                    .into(),
                ),
                recipient: None,
                result: None,
                sender: None,
                val: None,
            },
            fuel::Receipt::Revert {
                id: contract_id,
                ra,
                pc,
                is: isr,
            } => Self {
                amount: None,
                asset_id: None,
                contract_id: Some(contract_id),
                data: None,
                digest: None,
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: None,
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: None,
                ra: Some(ra),
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: None,
                result: None,
                sender: None,
                val: None,
            },
            fuel::Receipt::Log {
                id: contract_id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is: isr,
            } => Self {
                amount: None,
                asset_id: None,
                contract_id: Some(contract_id),
                data: None,
                digest: None,
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: None,
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: None,
                ra: Some(ra),
                rb: Some(rb),
                rc: Some(rc),
                rd: Some(rd),
                reason: None,
                recipient: None,
                result: None,
                sender: None,
                val: None,
            },
            fuel::Receipt::LogData {
                id: contract_id,
                ra,
                rb,
                ptr,
                len,
                digest,
                data,
                pc,
                is: isr,
            } => Self {
                amount: None,
                asset_id: None,
                contract_id: Some(contract_id),
                data: Some(data.into()),
                digest: Some(digest),
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: Some(len),
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: Some(ptr),
                ra: Some(ra),
                rb: Some(rb),
                rc: None,
                rd: None,
                reason: None,
                recipient: None,
                result: None,
                sender: None,
                val: None,
            },
            fuel::Receipt::Transfer {
                id: contract_id,
                to: recipient,
                amount,
                asset_id,
                pc,
                is: isr,
            } => Self {
                amount: Some(amount),
                asset_id: Some(asset_id),
                contract_id: Some(contract_id),
                data: None,
                digest: None,
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: None,
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: None,
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: Some(Identity::ContractId(recipient)),
                result: None,
                sender: None,
                val: None,
            },
            fuel::Receipt::TransferOut {
                id: contract_id,
                to: recipient,
                amount,
                asset_id,
                pc,
                is: isr,
            } => Self {
                amount: Some(amount),
                asset_id: Some(asset_id),
                contract_id: Some(contract_id),
                data: None,
                digest: None,
                gas_used: None,
                gas: None,
                isr: Some(isr),
                len: None,
                nonce: None,
                param1: None,
                param2: None,
                pc: Some(pc),
                ptr: None,
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: Some(Identity::Address(recipient)),
                result: None,
                sender: None,
                val: None,
            },
            fuel::Receipt::ScriptResult { result, gas_used } => Self {
                amount: None,
                asset_id: None,
                contract_id: None,
                data: None,
                digest: None,
                gas_used: Some(gas_used),
                gas: None,
                isr: None,
                len: None,
                nonce: None,
                param1: None,
                param2: None,
                pc: None,
                ptr: None,
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: None,
                result: Some(ScriptExecutionResult::from(result).into()),
                sender: None,
                val: None,
            },
            fuel::Receipt::MessageOut {
                sender,
                recipient,
                amount,
                nonce,
                len,
                digest,
                data,
            } => Self {
                amount: Some(amount),
                asset_id: None,
                contract_id: None,
                data: Some(data.into()),
                digest: Some(digest),
                gas_used: None,
                gas: None,
                isr: None,
                len: Some(len),
                nonce: Some(nonce),
                param1: None,
                param2: None,
                pc: None,
                ptr: None,
                ra: None,
                rb: None,
                rc: None,
                rd: None,
                reason: None,
                recipient: Some(Identity::Address(recipient)),
                result: None,
                sender: Some(sender),
                val: None,
            },
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
                        maturity: maturity.clone(),
                        script: script.to_owned().into(),
                        // storage_slots: [],
                        // inputs: [],
                        // inputs: [],
                        // outputs: [],
                        // witnesses: [],
                        receipts_root: *receipts_root,
                        metadata: metadata.to_owned().map(|m| m.into()),
                        is_script: true,
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
                        maturity: maturity.clone(),
                        bytecode_length: *bytecode_length,
                        bytecode_witness_index: *bytecode_witness_index,
                        // storage_slots: [],
                        // inputs: [],
                        // inputs: [],
                        // outputs: [],
                        // witnesses: [],
                        salt: *salt,
                        metadata: metadata.to_owned().map(|m| m.into()),
                        is_create: true,
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
                let _receipt_entity = Receipt::from(receipt.to_owned());
                match receipt {
                    fuel::Receipt::Call { .. } => {}
                    fuel::Receipt::ReturnData { .. } => {}
                    fuel::Receipt::Transfer { .. } => {}
                    fuel::Receipt::TransferOut { .. } => {}
                    fuel::Receipt::Log { .. } => {}
                    fuel::Receipt::LogData { .. } => {}
                    fuel::Receipt::ScriptResult { .. } => {}
                    fuel::Receipt::MessageOut { .. } => {}
                    _ => {
                        Logger::warn("This Receipt type is not handled yet.");
                    }
                }
            }
        }
    }
}
