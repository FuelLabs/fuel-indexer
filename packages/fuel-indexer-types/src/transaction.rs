use crate::fuel_client_schema::ClientTransactionStatusData;
pub use fuel_tx::{
    field::{
        BytecodeLength, BytecodeWitnessIndex, GasLimit, GasPrice, Inputs, Maturity,
        Outputs, ReceiptsRoot, Salt as TxFieldSalt, Script, ScriptData, StorageSlots,
        TxPointer as ClientFieldTxPointer, Witnesses,
    },
    Receipt as ClientReceipt, ScriptExecutionResult, Transaction as ClientTransaction,
    TxId, Witness as ClientWitness,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TransactionData {
    pub transaction: ClientTransaction,
    pub status: ClientTransactionStatusData,
    pub receipts: Vec<ClientReceipt>,
    pub id: TxId,
}
