use crate::scalar::{Address, AssetId, Bytes32, ContractId, HexString, Nonce};
pub use fuel_tx::UtxoId as ClientUtxoId;

// NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
pub enum ClientInput {
    Coin(ClientInputCoin),
    Contract(ClientInputContract),
    Message(ClientInputMessage),
}

// NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
pub struct ClientTxPointer {
    pub block_height: u32,
    pub tx_index: u64,
}

// NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
pub struct ClientInputCoin {
    pub utxo_id: ClientUtxoId,
    pub owner: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub tx_pointer: ClientTxPointer,
    pub witness_index: u8,
    pub maturity: u32,
    pub predicate: HexString,
    pub predicate_data: HexString,
}

// NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
pub struct ClientInputContract {
    pub utxo_id: ClientUtxoId,
    pub balance_root: Bytes32,
    pub state_root: Bytes32,
    pub tx_pointer: ClientTxPointer,
    pub contract_id: ContractId,
}

// NOTE: https://github.com/FuelLabs/fuel-indexer/issues/286
pub struct ClientInputMessage {
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Nonce,
    pub witness_index: u8,
    pub data: HexString,
    pub predicate: HexString,
    pub predicate_data: HexString,
}
