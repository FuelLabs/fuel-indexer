extern crate alloc;
use fuel_indexer_utils::prelude::*;

// TODO: We use a lot of manual type conversion below due to https://github.com/FuelLabs/fuel-indexer/issues/286

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
        Self::new(*block_height, tx_index).get_or_create()
    }
}

impl From<fuel::UtxoId> for UtxoId {
    fn from(utxo_id: fuel::UtxoId) -> Self {
        Self::new(*utxo_id.tx_id(), utxo_id.output_index() as u32).get_or_create()
    }
}

impl From<Bytes32> for ContractIdFragment {
    fn from(hash: Bytes32) -> Self {
        let hash = Bytes32::from(<[u8; 32]>::from(hash));
        Self::new(hash).get_or_create()
    }
}

impl From<ContractId> for ContractIdFragment {
    fn from(hash: ContractId) -> Self {
        let hash = Bytes32::from(<[u8; 32]>::from(hash));
        Self::new(hash).get_or_create()
    }
}

impl From<Bytes32> for BlockIdFragment {
    fn from(hash: Bytes32) -> Self {
        let hash = Bytes32::from(<[u8; 32]>::from(hash));
        Self::new(hash).get_or_create()
    }
}

impl From<Bytes32> for TransactionIdFragment {
    fn from(hash: Bytes32) -> Self {
        let hash = Bytes32::from(<[u8; 32]>::from(hash));
        Self::new(hash).get_or_create()
    }
}

impl From<fuel::TransactionData> for TransactionIdFragment {
    fn from(tx: fuel::TransactionData) -> Self {
        Self::from(Bytes32::from(<[u8; 32]>::from(tx.id))).get_or_create()
    }
}

impl From<fuel::StorageSlot> for StorageSlot {
    fn from(slot: fuel::StorageSlot) -> Self {
        let fuel::StorageSlot { key, value } = slot;
        Self::new(key, value).get_or_create()
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
                    *maturity,
                    predicate,
                    predicate_data,
                    InputLabel::Coin.into(),
                    true,
                );

                Self::from(input).get_or_create()
            }
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
                let contract = ContractIdFragment::from(contract_id);

                let input = InputContract::new(
                    utxo_id.id,
                    balance_root,
                    state_root,
                    tx_pointer.id,
                    contract.id,
                    InputLabel::Contract.into(),
                    true,
                );

                Self::from(input).get_or_create()
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
                    Bytes32::from(<[u8; 32]>::from(nonce)),
                    witness_index.into(),
                    data,
                    predicate,
                    predicate_data,
                    InputLabel::Message.into(),
                    true,
                );

                Self::from(input).get_or_create()
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

                Self::from(output).get_or_create()
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

                Self::from(output).get_or_create()
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

                Self::from(output).get_or_create()
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

                Self::from(output).get_or_create()
            }
            fuel::Output::ContractCreated(output) => {
                let fuel::ContractCreated {
                    contract_id,
                    state_root,
                } = output;

                let contract_frag = ContractIdFragment::from(contract_id);

                let output = ContractCreated::new(
                    contract_frag.id,
                    state_root,
                    true,
                    OutputLabel::ContractCreated.into(),
                );

                Self::from(output).get_or_create()
            }
            fuel::Output::Message(output) => {
                let fuel::MessageOutput { amount, recipient } = output;

                let output = MessageOutput::new(amount, recipient);

                Self::from(output)
            }
            fuel::Output::Unknown => {
                let output = UnknownOutput::new(true, OutputLabel::Unknown.into());

                Self::from(output).get_or_create()
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

                Self::from(status).get_or_create()
            }
            fuel::TransactionStatus::SqueezedOut { reason } => {
                let status = SqueezedOutStatus::new(
                    reason,
                    TransactionStatusLabel::SqueezedOut.into(),
                    true,
                );

                Self::from(status).get_or_create()
            }
            fuel::TransactionStatus::Failure {
                block,
                time,
                reason,
                program_state,
            } => {
                let block_id = BlockIdFragment::from(block);
                let program_state = program_state.map(|p| p.into());

                let status = FailureStatus::new(
                    block_id.id,
                    time,
                    reason,
                    program_state,
                    TransactionStatusLabel::Failure.into(),
                    true,
                );

                Self::from(status).get_or_create()
            }
            fuel::TransactionStatus::Success {
                block,
                time,
                program_state,
            } => {
                let block_id = BlockIdFragment::from(block);
                let program_state = program_state.map(|p| p.into());

                let status = SuccessStatus::new(
                    time,
                    block_id.id,
                    program_state,
                    TransactionStatusLabel::Success.into(),
                    true,
                );

                Self::from(status).get_or_create()
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
                    recipient: Identity::ContractId(<[u8; 32]>::from(recipient).into()),
                    amount,
                    asset_id: <[u8; 32]>::from(asset_id).into(),
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
                    ptr,
                    len,
                    digest: <[u8; 32]>::from(digest).into(),
                    data: data.map(|d| d.into()),
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
                let contract_id =
                    contract_id.map(|c| ContractId::from(<[u8; 32]>::from(c)));
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
                    ra,
                    rb,
                    ptr,
                    len,
                    digest: <[u8; 32]>::from(digest).into(),
                    data: data.map(|d| d.into()),
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
                    recipient: Identity::ContractId(<[u8; 32]>::from(recipient).into()),
                    amount,
                    asset_id: <[u8; 32]>::from(asset_id).into(),
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
                    contract_id: <[u8; 32]>::from(contract_id).into(),
                    recipient: Identity::Address(<[u8; 32]>::from(recipient).into()),
                    amount,
                    asset_id: <[u8; 32]>::from(asset_id).into(),
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
                ..
            } => {
                let receipt = MessageOutReceipt {
                    sender: <[u8; 32]>::from(sender).into(),
                    recipient: Identity::Address(<[u8; 32]>::from(recipient).into()),
                    amount,
                    nonce: <[u8; 32]>::from(nonce).into(),
                    len,
                    digest: <[u8; 32]>::from(digest).into(),
                    data: data.map(|d| d.into()),
                    label: ReceiptLabel::MessageOut.into(),
                    is_message_out: true,
                };

                Self::from(receipt)
            }
            fuel::Receipt::Mint {
                sub_id,
                contract_id,
                val,
                pc,
                is,
            } => {
                let receipt = MintReceipt {
                    sub_id,
                    contract_id,
                    val,
                    pc,
                    isr: is,
                };

                Self::from(receipt)
            }
            fuel::Receipt::Burn {
                sub_id,
                contract_id,
                val,
                pc,
                is,
            } => {
                let receipt = BurnReceipt {
                    sub_id,
                    contract_id,
                    val,
                    pc,
                    isr: is,
                };

                Self::from(receipt)
            }
        }
    }
}

impl From<fuel::TransactionData> for Transaction {
    fn from(transaction: fuel::TransactionData) -> Self {
        let tx_status = TransactionStatus::from(transaction.status.clone());
        match transaction.transaction {
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
                let inputs = inputs
                    .iter()
                    .map(|i| Input::from(i.to_owned()))
                    .map(|i| i.id)
                    .collect::<Vec<UID>>();
                let outputs = outputs
                    .iter()
                    .map(|o| Output::from(o.to_owned()))
                    .map(|o| o.id)
                    .collect::<Vec<UID>>();
                let witnesses = witnesses
                    .iter()
                    .map(|w| Witness::from(w.to_owned()))
                    .map(|w| w.into())
                    .collect::<Vec<_>>();

                let receipts = transaction
                    .receipts
                    .iter()
                    .map(|r| Receipt::from(r.to_owned()).into())
                    .collect::<Vec<_>>();

                let script_tx = ScriptTransaction::new(
                    gas_limit,
                    gas_price,
                    *maturity,
                    script.to_owned().into(),
                    script_data.to_owned().into(),
                    Some(inputs),
                    Some(outputs),
                    Some(witnesses),
                    receipts_root,
                    metadata.to_owned().map(|m| m.into()),
                    true,
                    Some(receipts),
                    tx_status.id,
                    TransactionLabel::Script.into(),
                );

                Self::from(script_tx).get_or_create()
            }
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
                let storage_slots = storage_slots
                    .iter()
                    .map(|s| StorageSlot::from(s.to_owned()))
                    .map(|s| s.id)
                    .collect::<Vec<UID>>();

                let inputs = inputs
                    .iter()
                    .map(|i| Input::from(i.to_owned()))
                    .map(|i| i.id)
                    .collect::<Vec<UID>>();
                let outputs = outputs
                    .iter()
                    .map(|o| Output::from(o.to_owned()))
                    .map(|o| o.id)
                    .collect::<Vec<UID>>();
                let witnesses = witnesses
                    .iter()
                    .map(|w| Witness::from(w.to_owned()))
                    .map(|w| w.into())
                    .collect::<Vec<_>>();
                let receipts = transaction
                    .receipts
                    .iter()
                    .map(|r| Receipt::from(r.to_owned()).into())
                    .collect::<Vec<_>>();

                let create_tx = CreateTransaction::new(
                    gas_limit,
                    gas_price,
                    *maturity,
                    bytecode_length,
                    bytecode_witness_index,
                    Some(storage_slots),
                    Some(inputs),
                    Some(outputs),
                    Some(witnesses),
                    to_bytes32(salt),
                    metadata.to_owned().map(|m| m.into()),
                    true,
                    Some(receipts),
                    tx_status.id,
                    TransactionLabel::Create.into(),
                );

                Self::from(create_tx).get_or_create()
            }
            fuel::Transaction::Mint(fuel::Mint {
                tx_pointer,
                outputs,
                metadata,
            }) => {
                let tx_pointer = TxPointer::from(tx_pointer.to_owned());
                let outputs = outputs
                    .iter()
                    .map(|o| Output::from(o.to_owned()))
                    .map(|o| o.id)
                    .collect::<Vec<UID>>();
                let receipts = transaction
                    .receipts
                    .iter()
                    .map(|r| Receipt::from(r.to_owned()).into())
                    .collect::<Vec<_>>();

                let mint_tx = MintTransaction::new(
                    tx_pointer.id,
                    Some(outputs),
                    metadata.to_owned().map(|m| m.into()),
                    true,
                    Some(receipts),
                    tx_status.id,
                    TransactionLabel::Mint.into(),
                );

                Self::from(mint_tx).get_or_create()
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
            block_data.header.message_receipt_count,
            block_data.header.transactions_root,
            block_data.header.message_receipt_root,
            block_data.header.height,
            block_data.header.prev_root,
            block_data.header.time,
            block_data.header.application_hash,
        );
        header.save();

        let block_frag = BlockIdFragment::from(block_data.header.id);

        let consensus = Consensus::from(block_data.consensus);
        consensus.save();

        let tx_id_frags = block_data
            .transactions
            .iter()
            .map(|t| TransactionIdFragment::from(t.to_owned()))
            .map(|t| t.id)
            .collect::<Vec<UID>>();

        let block = Block {
            id: block_frag.id,
            block_id: block_data.header.id,
            header: header.id,
            consensus: consensus.id,
            transactions: Some(tx_id_frags),
        };

        block.save();

        let _transactions = block_data
            .transactions
            .iter()
            .map(|t| Transaction::from(t.to_owned()))
            .collect::<Vec<Transaction>>();
    }
}
