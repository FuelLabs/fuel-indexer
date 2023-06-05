// ** All types in this file need to eventually be replace **
//
// TODO: https://github.com/FuelLabs/fuel-indexer/issues/286

use crate::{
    scalar::{
        Address, AssetId, BlockHeight, Bytes32, ContractId, HexString, Json, Salt,
        Signature,
    },
    type_id, TypeId, FUEL_TYPES_NAMESPACE,
};
pub use fuel_tx::{
    Input as ClientInput, Output as ClientOutput, Receipt, ScriptExecutionResult,
    Transaction as ClientTransaction, TxId, TxPointer as ClientTxPointer, UtxoId,
    Witness, Word,
};
use serde::{Deserialize, Serialize};

pub mod field {
    pub use fuel_tx::field::{
        BytecodeLength, BytecodeWitnessIndex, GasLimit, GasPrice, Inputs, Maturity,
        Outputs, ReceiptsRoot, Salt as TxFieldSalt, Script as TxFieldScript, ScriptData,
        StorageSlots, TxPointer as FieldTxPointer, Witnesses,
    };
}

pub type RawInstruction = u32;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StorageSlot {
    pub key: Bytes32,
    pub value: Bytes32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Transaction {
    Create(Create),
    Mint(Mint),
    Script(Script),
}

impl Default for Transaction {
    fn default() -> Self {
        Transaction::Script(Script::default())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Create {
    pub gas_price: Word,
    pub gas_limit: Word,
    pub maturity: BlockHeight,
    pub bytecode_length: Word,
    pub bytecode_witness_index: u8,
    pub storage_slots: Vec<StorageSlot>,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub witnesses: Vec<Witness>,
    pub salt: Salt,
    pub metadata: Option<CommonMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommonMetadata {
    pub id: Bytes32,
    pub inputs_offset: usize,
    pub inputs_offset_at: Vec<usize>,
    pub inputs_predicate_offset_at: Vec<Option<(usize, usize)>>,
    pub outputs_offset: usize,
    pub outputs_offset_at: Vec<usize>,
    pub witnesses_offset: usize,
    pub witnesses_offset_at: Vec<usize>,
}

impl From<CommonMetadata> for Json {
    fn from(metadata: CommonMetadata) -> Self {
        let s = serde_json::to_string(&metadata).expect("Serde error.");
        Self(s)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Script {
    pub gas_price: Word,
    pub gas_limit: Word,
    pub maturity: BlockHeight,
    pub script: Vec<u8>,
    pub script_data: Vec<u8>,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub witnesses: Vec<Witness>,
    pub receipts_root: Bytes32,
    pub metadata: Option<ScriptMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub common: CommonMetadata,
    pub script_data_offset: usize,
}

impl From<ScriptMetadata> for Json {
    fn from(metadata: ScriptMetadata) -> Self {
        let s = serde_json::to_string(&metadata).expect("Serde error.");
        Self(s)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mint {
    pub tx_pointer: TxPointer,
    pub outputs: Vec<Output>,
    pub metadata: Option<MintMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MintMetadata {
    pub id: Bytes32,
    pub outputs_offset: usize,
    pub outputs_offset_at: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub transaction: Transaction,
    pub status: TransactionStatus,
    pub receipts: Vec<Receipt>,
    pub id: TxId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub id: Bytes32,
    pub da_height: u64,
    pub transactions_count: u64,
    pub output_messages_count: u64,
    pub transactions_root: Bytes32,
    pub output_messages_root: Bytes32,
    pub height: u64,
    pub prev_root: Bytes32,
    pub time: i64,
    pub application_hash: Bytes32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub height: u64,
    pub id: Bytes32,
    pub header: Header,
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub consensus: Consensus,
    pub transactions: Vec<TransactionData>,
}

impl TypeId for BlockData {
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "BlockData") as usize
    }
}

impl From<ClientTxPointer> for TxPointer {
    fn from(tx_pointer: ClientTxPointer) -> Self {
        TxPointer {
            block_height: tx_pointer.block_height(),
            tx_index: tx_pointer.tx_index() as u64,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    Coin(InputCoin),
    Contract(InputContract),
    Message(InputMessage),
}

impl From<ClientInput> for Input {
    fn from(input: ClientInput) -> Self {
        match input {
            ClientInput::CoinSigned {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer,
                witness_index,
                maturity,
            } => Input::Coin(InputCoin {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer: tx_pointer.into(),
                witness_index,
                maturity,
                predicate: "".into(),
                predicate_data: "".into(),
            }),
            ClientInput::CoinPredicate {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer,
                maturity,
                predicate,
                predicate_data,
            } => Input::Coin(InputCoin {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer: tx_pointer.into(),
                witness_index: 0,
                maturity,
                predicate: predicate.into(),
                predicate_data: predicate_data.into(),
            }),
            ClientInput::Contract {
                utxo_id,
                balance_root,
                state_root,
                tx_pointer,
                contract_id,
            } => Input::Contract(InputContract {
                utxo_id,
                balance_root,
                state_root,
                tx_pointer: tx_pointer.into(),
                contract_id,
            }),
            ClientInput::MessageSigned {
                amount,
                witness_index,
                sender,
                recipient,
                nonce,
                data,
                ..
            } => Input::Message(InputMessage {
                amount,
                nonce,
                recipient,
                sender,
                witness_index,
                data: data.into(),
                predicate: "".into(),
                predicate_data: "".into(),
            }),
            ClientInput::MessagePredicate {
                amount,
                predicate,
                predicate_data,
                sender,
                recipient,
                nonce,
                data,
                ..
            } => Input::Message(InputMessage {
                sender,
                recipient,
                amount,
                nonce,
                witness_index: 0,
                data: data.into(),
                predicate: predicate.into(),
                predicate_data: predicate_data.into(),
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxPointer {
    pub block_height: u32,
    pub tx_index: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputCoin {
    pub utxo_id: UtxoId,
    pub owner: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub tx_pointer: TxPointer,
    pub witness_index: u8,
    pub maturity: u64,
    pub predicate: HexString,
    pub predicate_data: HexString,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputContract {
    pub utxo_id: UtxoId,
    pub balance_root: Bytes32,
    pub state_root: Bytes32,
    pub tx_pointer: TxPointer,
    pub contract_id: ContractId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputMessage {
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: u64,
    pub witness_index: u8,
    pub data: HexString,
    pub predicate: HexString,
    pub predicate_data: HexString,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionStatus {
    Failure {
        block: Bytes32,
        time: u64,
        reason: String,
        program_state: Option<ProgramState>,
    },
    SqueezedOut {
        reason: String,
    },
    Submitted {
        submitted_at: u64,
    },
    Success {
        block: Bytes32,
        time: u64,
        program_state: Option<ProgramState>,
    },
}

impl Default for TransactionStatus {
    fn default() -> Self {
        TransactionStatus::SqueezedOut {
            reason: "default".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractIdFragment {
    pub id: ContractId,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Output {
    CoinOutput(CoinOutput),
    ContractOutput(ContractOutput),
    ChangeOutput(ChangeOutput),
    VariableOutput(VariableOutput),
    ContractCreated(ContractCreated),
    #[default]
    Unknown,
}

impl From<ClientOutput> for Output {
    fn from(output: ClientOutput) -> Self {
        match output {
            ClientOutput::Coin {
                to,
                amount,
                asset_id,
            } => Output::CoinOutput(CoinOutput {
                to,
                amount,
                asset_id,
            }),
            ClientOutput::Contract {
                input_index,
                balance_root,
                state_root,
            } => Output::ContractOutput(ContractOutput {
                input_index: input_index.into(),
                balance_root,
                state_root,
            }),
            ClientOutput::Change {
                to,
                amount,
                asset_id,
            } => Output::ChangeOutput(ChangeOutput {
                to,
                amount,
                asset_id,
            }),
            ClientOutput::Variable {
                to,
                amount,
                asset_id,
            } => Output::VariableOutput(VariableOutput {
                to,
                amount,
                asset_id,
            }),
            ClientOutput::ContractCreated {
                contract_id,
                state_root,
            } => Output::ContractCreated(ContractCreated {
                contract_id,
                state_root,
            }),
            _ => Output::Unknown,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractOutput {
    pub input_index: i32,
    pub balance_root: Bytes32,
    pub state_root: Bytes32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariableOutput {
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractCreated {
    pub contract_id: ContractId,
    pub state_root: Bytes32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genesis {
    pub chain_config_hash: Bytes32,
    pub coins_root: Bytes32,
    pub contracts_root: Bytes32,
    pub messages_root: Bytes32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoA {
    pub signature: Signature,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Consensus {
    Genesis(Genesis),
    PoA(PoA),
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReturnType {
    Return,
    ReturnData,
    Revert,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramState {
    pub return_type: ReturnType,
    pub data: HexString,
}

impl From<ProgramState> for Json {
    fn from(state: ProgramState) -> Self {
        let s = serde_json::to_string(&state).expect("Serde error.");
        Self(s)
    }
}

impl From<Json> for ProgramState {
    fn from(json: Json) -> Self {
        let state: ProgramState = serde_json::from_str(&json.0).expect("Serde error.");
        state
    }
}

pub enum PanicReason {
    /// 0 is reserved for success, while any non-zero value indicates a failure.
    Success = 0x00,
    /// Found `RVRT` instruction.
    Revert = 0x01,
    /// Execution ran out of gas.
    OutOfGas = 0x02,
    /// The transaction validity is violated.
    TransactionValidity = 0x03,
    /// Attempt to write outside interpreter memory boundaries.
    MemoryOverflow = 0x04,
    /// Overflow while executing arithmetic operation.
    /// These errors are ignored using the WRAPPING flag.
    ArithmeticOverflow = 0x05,
    /// Designed contract was not found in the storage.
    ContractNotFound = 0x06,
    /// Memory ownership rules are violated.
    MemoryOwnership = 0x07,
    /// The asset ID balance isn't enough for the instruction.
    NotEnoughBalance = 0x08,
    /// The interpreter is expected to be in internal context.
    ExpectedInternalContext = 0x09,
    /// The queried asset ID was not found in the state.
    AssetIdNotFound = 0x0a,
    /// The provided input is not found in the transaction.
    InputNotFound = 0x0b,
    /// The provided output is not found in the transaction.
    OutputNotFound = 0x0c,
    /// The provided witness is not found in the transaction.
    WitnessNotFound = 0x0d,
    /// The transaction maturity is not valid for this request.
    TransactionMaturity = 0x0e,
    /// The metadata identifier is invalid.
    InvalidMetadataIdentifier = 0x0f,
    /// The call structure is not valid.
    MalformedCallStructure = 0x10,
    /// The provided register does not allow write operations.
    ReservedRegisterNotWritable = 0x11,
    /// The execution resulted in an erroneous state of the interpreter.
    ErrorFlag = 0x12,
    /// The provided immediate value is not valid for this instruction.
    InvalidImmediateValue = 0x13,
    /// The provided transaction input is not of type `Coin`.
    ExpectedCoinInput = 0x14,
    /// The requested memory access exceeds the limits of the interpreter.
    MaxMemoryAccess = 0x15,
    /// Two segments of the interpreter memory should not intersect for write operations.
    MemoryWriteOverlap = 0x16,
    /// The requested contract is not listed in the transaction inputs.
    ContractNotInInputs = 0x17,
    /// The internal asset ID balance overflowed with the provided instruction.
    InternalBalanceOverflow = 0x18,
    /// The maximum allowed contract size is violated.
    ContractMaxSize = 0x19,
    /// This instruction expects the stack area to be unallocated for this call.
    ExpectedUnallocatedStack = 0x1a,
    /// The maximum allowed number of static contracts was reached for this transaction.
    MaxStaticContractsReached = 0x1b,
    /// The requested transfer amount cannot be zero.
    TransferAmountCannotBeZero = 0x1c,
    /// The provided transaction output should be of type `Variable`.
    ExpectedOutputVariable = 0x1d,
    /// The expected context of the stack parent is internal.
    ExpectedParentInternalContext = 0x1e,
    /// The jump instruction cannot move backwards in predicate verification.
    IllegalJump = 0x1f,
    /// The contract ID is already deployed and can't be overwritten.
    ContractIdAlreadyDeployed = 0x20,
    /// The loaded contract mismatch expectations.
    ContractMismatch = 0x21,
    /// Attempting to send message data longer than `MAX_MESSAGE_DATA_LENGTH`
    MessageDataTooLong = 0x22,
    /// Mathimatically invalid arguments where given to an arithmetic instruction.
    /// For instance, division by zero produces this.
    /// These errors are ignored using the UNSAFEMATH flag.
    ArithmeticError = 0x23,
    /// The byte can't be mapped to any known `PanicReason`.
    UnknownPanicReason = 0x24,
}

#[allow(unused)]
pub struct InstructionResult {
    reason: PanicReason,
    instruction: RawInstruction,
}
