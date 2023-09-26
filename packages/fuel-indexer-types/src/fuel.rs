// ** All types in this file need to eventually be replace **
//
// TODO: https://github.com/FuelLabs/fuel-indexer/issues/286

use crate::{scalar::*, type_id, TypeId, FUEL_TYPES_NAMESPACE};
pub use fuel_tx::ScriptExecutionResult;
pub use fuel_tx::{
    Input as ClientInput, Output as ClientOutput, PanicReason as ClientPanicReason,
    Transaction as ClientTransaction, TxPointer as ClientTxPointer,
};
pub use fuel_tx::{Receipt, TxId, UtxoId, Witness, Word};
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
        let s = serde_json::to_string(&metadata)
            .expect("Failed to serialize CommonMetadata.");
        Self::new(s)
    }
}

impl From<Json> for CommonMetadata {
    fn from(json: Json) -> Self {
        let metadata: CommonMetadata = serde_json::from_str(&json.into_inner())
            .expect("Failed to deserialize CommonMetadata.");
        metadata
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
        let s = serde_json::to_string(&metadata)
            .expect("Failed to deserialize MintMetadata.");
        Self::new(s)
    }
}

impl From<Json> for ScriptMetadata {
    fn from(json: Json) -> Self {
        let metadata: ScriptMetadata = serde_json::from_str(&json.into_inner())
            .expect("Failed to deserialize ScriptMetadata.");
        metadata
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

impl From<MintMetadata> for Json {
    fn from(metadata: MintMetadata) -> Self {
        let s =
            serde_json::to_string(&metadata).expect("Failed to serialize MintMetadata.");
        Self::new(s)
    }
}

impl From<Json> for MintMetadata {
    fn from(json: Json) -> Self {
        let metadata: MintMetadata = serde_json::from_str(&json.into_inner())
            .expect("Failed to deserialize MintMetadata.");
        metadata
    }
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
    pub message_receipt_count: u64,
    pub transactions_root: Bytes32,
    pub message_receipt_root: Bytes32,
    pub height: u32,
    pub prev_root: Bytes32,
    pub time: i64,
    pub application_hash: Bytes32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub height: u32,
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
            ClientInput::MessageDataSigned(message_signed) => {
                Input::Message(InputMessage {
                    sender: Address::from(<[u8; 32]>::from(message_signed.sender)),
                    recipient: Address::from(<[u8; 32]>::from(message_signed.recipient)),
                    amount: message_signed.amount,
                    nonce: message_signed.nonce,
                    witness_index: message_signed.witness_index,
                    data: message_signed.data,
                    predicate: Bytes::new(),
                    predicate_data: Bytes::new(),
                })
            }
            ClientInput::MessageDataPredicate(message_predicate) => {
                Input::Message(InputMessage {
                    sender: Address::from(<[u8; 32]>::from(message_predicate.sender)),
                    recipient: Address::from(<[u8; 32]>::from(
                        message_predicate.recipient,
                    )),
                    amount: message_predicate.amount,
                    nonce: message_predicate.nonce,
                    witness_index: 0,
                    data: message_predicate.data,
                    predicate: message_predicate.predicate,
                    predicate_data: message_predicate.predicate_data,
                })
            }
            ClientInput::CoinSigned(coin_signed) => Input::Coin(InputCoin {
                utxo_id: coin_signed.utxo_id,
                owner: Address::from(<[u8; 32]>::from(coin_signed.owner)),
                amount: coin_signed.amount,
                asset_id: AssetId::from(<[u8; 32]>::from(coin_signed.asset_id)),
                tx_pointer: coin_signed.tx_pointer.into(),
                witness_index: coin_signed.witness_index,
                maturity: coin_signed.maturity,
                predicate: Bytes::new(),
                predicate_data: Bytes::new(),
            }),
            ClientInput::CoinPredicate(coin_predicate) => Input::Coin(InputCoin {
                utxo_id: coin_predicate.utxo_id,
                owner: Address::from(<[u8; 32]>::from(coin_predicate.owner)),
                amount: coin_predicate.amount,
                asset_id: AssetId::from(<[u8; 32]>::from(coin_predicate.asset_id)),
                tx_pointer: coin_predicate.tx_pointer.into(),
                witness_index: 0,
                maturity: coin_predicate.maturity,
                predicate: coin_predicate.predicate,
                predicate_data: coin_predicate.predicate_data,
            }),
            ClientInput::Contract(contract) => Input::Contract(InputContract {
                utxo_id: contract.utxo_id,
                balance_root: contract.balance_root,
                state_root: contract.state_root,
                tx_pointer: contract.tx_pointer.into(),
                contract_id: ContractId::from(<[u8; 32]>::from(contract.contract_id)),
            }),
            ClientInput::MessageCoinSigned(message_coin) => {
                Input::Message(InputMessage {
                    sender: Address::from(<[u8; 32]>::from(message_coin.sender)),
                    recipient: Address::from(<[u8; 32]>::from(message_coin.recipient)),
                    amount: message_coin.amount,
                    nonce: message_coin.nonce,
                    witness_index: message_coin.witness_index,
                    data: Bytes::new(),
                    predicate: Bytes::new(),
                    predicate_data: Bytes::new(),
                })
            }
            ClientInput::MessageCoinPredicate(message_coin) => {
                Input::Message(InputMessage {
                    sender: Address::from(<[u8; 32]>::from(message_coin.sender)),
                    recipient: Address::from(<[u8; 32]>::from(message_coin.recipient)),
                    amount: message_coin.amount,
                    nonce: message_coin.nonce,
                    witness_index: 0,
                    data: Bytes::new(),
                    predicate: message_coin.predicate,
                    predicate_data: message_coin.predicate_data,
                })
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxPointer {
    pub block_height: BlockHeight,
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
    pub maturity: BlockHeight,
    pub predicate: Bytes,
    pub predicate_data: Bytes,
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
    pub nonce: Nonce,
    pub witness_index: u8,
    pub data: Bytes,
    pub predicate: Bytes,
    pub predicate_data: Bytes,
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
    Message(MessageOutput),
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
                to: Address::from(
                    <[u8; 32]>::try_from(to).expect("Could not convert 'to' to bytes"),
                ),
                amount,
                asset_id: AssetId::from(
                    <[u8; 32]>::try_from(asset_id)
                        .expect("Could not convert asset ID to bytes"),
                ),
            }),
            ClientOutput::Contract {
                input_index,
                balance_root,
                state_root,
            } => Output::ContractOutput(ContractOutput {
                input_index: input_index.into(),
                balance_root: Bytes32::from(
                    <[u8; 32]>::try_from(balance_root)
                        .expect("Could not convert balance root to bytes"),
                ),
                state_root: Bytes32::from(
                    <[u8; 32]>::try_from(state_root)
                        .expect("Could not convert state root to bytes"),
                ),
            }),
            ClientOutput::Change {
                to,
                amount,
                asset_id,
            } => Output::ChangeOutput(ChangeOutput {
                to: Address::from(
                    <[u8; 32]>::try_from(to).expect("Could not convert 'to' to bytes"),
                ),
                amount,
                asset_id: AssetId::from(
                    <[u8; 32]>::try_from(asset_id)
                        .expect("Could not convert asset ID to bytes"),
                ),
            }),
            ClientOutput::Variable {
                to,
                amount,
                asset_id,
            } => Output::VariableOutput(VariableOutput {
                to: Address::from(
                    <[u8; 32]>::try_from(to).expect("Could not convert 'to' to bytes"),
                ),
                amount,
                asset_id: AssetId::from(
                    <[u8; 32]>::try_from(asset_id)
                        .expect("Could not convert asset ID to bytes"),
                ),
            }),
            ClientOutput::ContractCreated {
                contract_id,
                state_root,
            } => Output::ContractCreated(ContractCreated {
                contract_id: ContractId::from(
                    <[u8; 32]>::try_from(contract_id)
                        .expect("Could not convert contract ID to bytes"),
                ),
                state_root: Bytes32::from(
                    <[u8; 32]>::try_from(state_root)
                        .expect("Could not convert state root to bytes"),
                ),
            }),
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
pub struct MessageOutput {
    pub amount: u64,
    pub recipient: Address,
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
    pub signature: Bytes64,
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
    pub data: Bytes,
}

impl From<ProgramState> for Json {
    fn from(state: ProgramState) -> Self {
        let s = serde_json::to_string(&state).expect("Failed to serialize ProgramState.");
        Self::new(s)
    }
}

impl From<Json> for ProgramState {
    fn from(json: Json) -> Self {
        let state: ProgramState = serde_json::from_str(&json.into_inner())
            .expect("Failed to deserialize ProgramState.");
        state
    }
}
