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
                true,
                ConsensusLabel::Genesis.into(),
            )),
            fuel::Consensus::PoA(poa) => {
                Consensus::from(PoA::new(poa.signature, true, ConsensusLabel::PoA.into()))
            }
            fuel::Consensus::Unknown => {
                Consensus::from(Unknown::new(true, ConsensusLabel::Unknown.into()))
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

                let input = InputCoin::new(
                    utxo_id.id,
                    owner,
                    amount,
                    asset_id,
                    tx_pointer.id,
                    witness_index.into(),
                    maturity,
                    predicate.into(),
                    predicate_data.into(),
                    InputLabel::Coin.into(),
                    true,
                );

                Self::from(input)
            }
            #[allow(unused)]
            fuel::Input::Contract(input) => {
                let fuel::InputContract {
                    utxo_id,
                    balance_root,
                    state_root,
                    tx_pointer,
                    contract_id,
                } = input;

                let utxo_id = UtxoId::from(utxo_id);
                let tx_pointer = TxPointer::from(tx_pointer);

                let input = InputContract::new(
                    utxo_id.id,
                    balance_root,
                    state_root,
                    tx_pointer.id,
                    id8(contract_id),
                    InputLabel::Contract.into(),
                    true,
                );

                Self::from(input)
            }
            fuel::Input::Message(input) => {
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

                let input = InputMessage::new(
                    sender,
                    recipient,
                    amount,
                    nonce,
                    witness_index.into(),
                    data.into(),
                    predicate.into(),
                    predicate_data.into(),
                    InputLabel::Message.into(),
                    true,
                );

                Self::from(input)
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

                let output =
                    CoinOutput::new(to, amount, asset_id, true, OutputLabel::Coin.into());

                Self::from(output)
            }
            fuel::Output::ContractOutput(output) => {
                let fuel::ContractOutput {
                    input_index,
                    balance_root,
                    state_root,
                } = output;

                let output = ContractOutput::new(
                    input_index.into(),
                    balance_root,
                    state_root,
                    true,
                    OutputLabel::Contract.into(),
                );

                Self::from(output)
            }
            fuel::Output::ChangeOutput(output) => {
                let fuel::ChangeOutput {
                    to,
                    amount,
                    asset_id,
                } = output;
                let output = ChangeOutput::new(
                    to,
                    amount,
                    asset_id,
                    true,
                    OutputLabel::Change.into(),
                );

                Self::from(output)
            }
            fuel::Output::VariableOutput(output) => {
                let fuel::VariableOutput {
                    to,
                    amount,
                    asset_id,
                } = output;

                let output = VariableOutput::new(
                    to,
                    amount,
                    asset_id,
                    true,
                    OutputLabel::Variable.into(),
                );

                Self::from(output)
            }
            fuel::Output::ContractCreated(output) => {
                let fuel::ContractCreated {
                    contract_id,
                    state_root,
                } = output;

                let output = ContractCreated::new(
                    id8(contract_id),
                    state_root,
                    OutputLabel::ContractCreated.into(),
                    true,
                );

                Self::from(output)
            }
            fuel::Output::Unknown => {
                let output = UnknownOutput::new(OutputLabel::Unknown.into(), true);

                Self::from(output)
            }
        }
    }
}

impl From<fuel::TransactionStatus> for TransactionStatus {
    fn from(status: fuel::TransactionStatus) -> Self {
        match status {
            fuel::TransactionStatus::Submitted { submitted_at } => {
                let status = SubmittedStatus::new(
                    submitted_at,
                    TransactionStatusLabel::Submitted.into(),
                    true,
                );
                Self::from(status)
            }
            fuel::TransactionStatus::SqueezedOut { reason } => {
                let status = SqueezedOutStatus::new(
                    reason,
                    TransactionStatusLabel::SqueezedOut.into(),
                    true,
                );

                Self::from(status)
            }
            #[allow(unused)]
            fuel::TransactionStatus::Failure {
                block,
                time,
                reason,
                program_state,
            } => {
                // TODO: Create UID here.
                let block_id = 1;
                let program_state = program_state.map(|p| p.into());

                let status = FailureStatus::new(
                    block_id,
                    time,
                    reason,
                    program_state,
                    TransactionStatusLabel::Failure.into(),
                    true,
                );

                Self::from(status)
            }
            #[allow(unused)]
            fuel::TransactionStatus::Success {
                block,
                time,
                program_state,
            } => {
                // TODO: Create UID here.
                let block_id = 1;
                let program_state = program_state.map(|p| p.into());

                let status = SuccessStatus::new(
                    block_id,
                    time,
                    program_state,
                    TransactionStatusLabel::Success.into(),
                    true,
                );

                Self::from(status)
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
            } => {
                let receipt = CallReceipt {
                    contract_id,
                    recipient: Identity::ContractId(recipient),
                    amount,
                    asset_id,
                    gas,
                    param1,
                    param2,
                    pc,
                    isr,
                    label: ReceiptLabel::Call.into(),
                    is_call: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::Return {
                id: contract_id,
                val,
                pc,
                is: isr,
            } => {
                let receipt = ReturnReceipt {
                    contract_id,
                    val,
                    pc,
                    isr,
                    label: ReceiptLabel::Return.into(),
                    is_return: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::ReturnData {
                id: contract_id,
                ptr,
                len,
                digest,
                data,
                pc,
                is: isr,
            } => {
                let receipt = ReturnDataReceipt {
                    contract_id,
                    ptr,
                    len,
                    digest,
                    data: data.into(),
                    pc,
                    isr,
                    label: ReceiptLabel::ReturnData.into(),
                    is_return_data: true,
                };

                Self::from(receipt)
            }
            // TODO: What to do with this id?
            #[allow(unused)]
            fuel::Receipt::Panic {
                id,
                contract_id,
                reason,
                pc,
                is: isr,
            } => {
                let receipt = PanicReceipt {
                    contract_id,
                    reason: Some(
                        InstructionResult {
                            reason: PanicReason::from(reason.reason().to_owned()).into(),
                            instruction: *reason.instruction(),
                        }
                        .into(),
                    ),
                    pc,
                    isr,
                    label: ReceiptLabel::Panic.into(),
                    is_panic: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::Revert {
                id: contract_id,
                ra,
                pc,
                is: isr,
            } => {
                let receipt = RevertReceipt {
                    contract_id,
                    ra,
                    pc,
                    isr,
                    label: ReceiptLabel::Revert.into(),
                    is_revert: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::Log {
                id: contract_id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is: isr,
            } => {
                let receipt = LogReceipt {
                    contract_id,
                    ra,
                    rb,
                    rc,
                    rd,
                    pc,
                    isr,
                    label: ReceiptLabel::Log.into(),
                    is_log: true,
                };

                Self::from(receipt)
            }
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
            } => {
                let receipt = LogDataReceipt {
                    contract_id,
                    ra,
                    rb,
                    ptr,
                    len,
                    digest,
                    data: data.into(),
                    pc,
                    isr,
                    label: ReceiptLabel::LogData.into(),
                    is_log_data: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::Transfer {
                id: contract_id,
                to: recipient,
                amount,
                asset_id,
                pc,
                is: isr,
            } => {
                let receipt = TransferReceipt {
                    contract_id,
                    recipient: Identity::ContractId(recipient),
                    amount,
                    asset_id,
                    pc,
                    isr,
                    label: ReceiptLabel::Transfer.into(),
                    is_transfer: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::TransferOut {
                id: contract_id,
                to: recipient,
                amount,
                asset_id,
                pc,
                is: isr,
            } => {
                let receipt = TransferOutReceipt {
                    contract_id,
                    recipient: Identity::Address(recipient),
                    amount,
                    asset_id,
                    pc,
                    isr,
                    label: ReceiptLabel::TransferOut.into(),
                    is_transfer_out: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::ScriptResult { result, gas_used } => {
                let receipt = ScriptResultReceipt {
                    result: ScriptExecutionResult::from(result).into(),
                    gas_used,
                    label: ReceiptLabel::ScriptResult.into(),
                    is_script_result: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::MessageOut {
                sender,
                recipient,
                amount,
                nonce,
                len,
                digest,
                data,
            } => {
                let receipt = MessageOutReceipt {
                    sender,
                    recipient: Identity::Address(recipient),
                    amount,
                    nonce,
                    len,
                    digest,
                    data: data.into(),
                    label: ReceiptLabel::MessageOut.into(),
                    is_message_out: true,
                };

                Self::from(receipt)
            }
        }
    }
}

#[indexer(manifest = "examples/fuel-explorer/fuel-explorer/fuel_explorer.manifest.yaml")]
pub mod explorer_index {

    fn index_block(block_data: BlockData) {
        let header = Header::new(
            block_data.header.id,
            block_data.header.da_height,
            block_data.header.transactions_count,
            block_data.header.output_messages_count,
            block_data.header.transactions_root,
            block_data.header.output_messages_root,
            block_data.header.height,
            block_data.header.prev_root,
            block_data.header.time,
            block_data.header.application_hash,
        );
        header.save();

        let consensus = Consensus::from(block_data.consensus);
        consensus.save();

        let block = Block {
            id: first8_bytes_to_u64(block_data.header.id),
            block_id: block_data.header.id,
            header: header.id,
            consensus: consensus.id,
        };

        // Save partial block
        block.save();

        let block_frag = BlockIdFragment {
            id: first8_bytes_to_u64(block_data.header.id),
            hash: Bytes32::default(),
        };

        block_frag.save();

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
