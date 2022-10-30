# Block Explorer

An extremely basic block explorer backend implementation that shows how to leverage basic Fuel indexer abstractions in order to build a cool dApp backend.

```rust
//! An extremely basic block explorer implementation that shows how blocks, transactions,
//! contracts, and accounts can be persisted into the database.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p explorer-index --release
//! ```
//!
//! Use the fuel-indexer testing components to start your Fuel node and web API
//!
//! ```bash
//! bash scripts/utils/start_test_components.bash
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- --manifest examples/block-explorer/manifest.yaml
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::{
    types::{Bytes32, ContractId},
    utils::sha256_digest,
};
use std::collections::HashSet;

// Using an ID for some abstraction (Contract, Account, etc), naively derive
// a unique ID for some database entity
pub fn derive_id(id: Vec<u8>, data: Vec<u8>) -> Bytes32 {
    let mut buff: [u8; 32] = [0u8; 32];
    let result = [id, data].concat();
    buff.copy_from_slice(&sha256_digest(&result).as_bytes()[..32]);
    Bytes32::from(buff)
}

// We'll pass our manifest to our #[indexer] attribute. This manifest contains
// all of the relevant configuration parameters in regard to how our index will
// work. In the fuel-indexer repository, we use relative paths (starting from the
// fuel-indexer root) but if you're building an index outside of the fuel-indexer
// project you'll want to use full/absolute paths.
#[indexer(manifest = "examples/block-explorer/manifest.yaml")]
mod explorer_index {

    // When specifying args to your handler functions, you can either use types defined
    // in your ABI JSON file, or you can use native Fuel types. These native Fuel types
    // include various `Receipt`s, as well as more comprehensive data, in the form of
    // `BlockData`. A list of native Fuel types can be found at [TODO INSERT LINK]
    fn index_explorer_data(block: BlockData) {
        // Convert the `BlockData` struct that we get from our Fuel node, into
        // a block entity that we can persist to the database. The `Block` type below is
        // defined in our schema/explorer.graphql and represents the type that we will
        // save to our database.
        let mut block_gas_limit = 0;

        let blck = Block {
            id: block.id,
            height: block.height,
            timestamp: block.time,
            miner: block.producer,
            gas_limit: block_gas_limit,
        };

        // Now that we've created the object for the database, let's save it.
        blck.save();

        // Keep track of some Receipt data involved in this transaction.
        let mut accounts = HashSet::new();
        let mut contracts = HashSet::new();

        // Now we'll iterate over all of the transactions in this block, and persist
        // those to the database as well
        for tx in block.transactions.iter() {
            let mut tx_amount = 0;
            let mut tokens_transferred = Vec::new();

            // Here we demonstrate that we can inspect the innards of the Transaction enum
            // for properties like gas, inputs, outputs, script_data, and other pieces of metadata.
            match &tx.transaction {
                #[allow(unused)]
                Transaction::Script {
                    gas_price,
                    gas_limit,
                    maturity,
                    receipts_root,
                    script,
                    script_data,
                    inputs,
                    outputs,
                    witnesses,
                    metadata,
                } => {
                    Logger::info("Inside a script transaction. (>^‿^)>");
                    block_gas_limit += gas_limit;
                }
                #[allow(unused)]
                Transaction::Create {
                    gas_price,
                    gas_limit,
                    maturity,
                    bytecode_length,
                    bytecode_witness_index,
                    salt,
                    storage_slots,
                    inputs,
                    outputs,
                    witnesses,
                    metadata,
                } => {
                    Logger::info("Inside a create transaction. <(^.^)>");
                    block_gas_limit += gas_limit;
                }
            }

            for receipt in &tx.receipts {
                // Here we can handle each receipt in a transaction as we like, the
                // code below demonstrates how you can use parts of a receipt in order
                // to persist entities to the database.
                match receipt {
                    #[allow(unused)]
                    Receipt::Call { id, .. } => {
                        contracts.insert(Contract {
                            id: *id,
                            last_seen: 0,
                        });
                    }
                    #[allow(unused)]
                    Receipt::ReturnData { id, .. } => {
                        contracts.insert(Contract {
                            id: *id,
                            last_seen: 0,
                        });
                    }
                    #[allow(unused)]
                    Receipt::Transfer {
                        id,
                        to,
                        asset_id,
                        amount,
                        ..
                    } => {
                        contracts.insert(Contract {
                            id: *id,
                            last_seen: 0,
                        });

                        let transfer = Transfer {
                            contract_id: *id,
                            receiver: *to,
                            amount: *amount,
                            asset_id: *asset_id,
                        };

                        transfer.save();
                        tokens_transferred.push(asset_id.to_string());
                    }
                    #[allow(unused)]
                    Receipt::TransferOut {
                        id,
                        to,
                        amount,
                        asset_id,
                        ..
                    } => {
                        contracts.insert(Contract {
                            id: *id,
                            last_seen: 0,
                        });

                        accounts.insert(Account {
                            id: *to,
                            last_seen: 0,
                        });

                        tx_amount += amount;
                        let transfer_out = TransferOut {
                            contract_id: *id,
                            receiver: *to,
                            amount: *amount,
                            asset_id: *asset_id,
                        };

                        transfer_out.save();
                    }
                    #[allow(unused)]
                    Receipt::Log { id, rb, .. } => {
                        contracts.insert(Contract {
                            id: *id,
                            last_seen: 0,
                        });

                        let id = derive_id(id.to_vec(), u64::to_le_bytes(*rb).to_vec());
                        let log = Log {
                            id,
                            contract_id: ContractId::from(*id),
                            rb: *rb,
                        };

                        log.save();
                    }
                    #[allow(unused)]
                    Receipt::LogData { id, .. } => {
                        contracts.insert(Contract {
                            id: *id,
                            last_seen: 0,
                        });

                        Logger::info("LogData types are unused in this example. (>'')>");
                    }
                    #[allow(unused)]
                    Receipt::ScriptResult { result, gas_used } => {
                        let result: u64 = match result {
                            ScriptExecutionResult::Success => 1,
                            ScriptExecutionResult::Revert => 2,
                            ScriptExecutionResult::Panic => 3,
                            ScriptExecutionResult::GenericFailure(_) => 4,
                        };
                        let r = ScriptResult {
                            id: derive_id(
                                [0u8; 32].to_vec(),
                                u64::to_be_bytes(result).to_vec(),
                            ),
                            result,
                            gas_used: *gas_used,
                        };
                        r.save();
                    }
                    #[allow(unused)]
                    Receipt::MessageOut {
                        sender,
                        recipient,
                        amount,
                        ..
                    } => {
                        tx_amount += amount;
                        accounts.insert(Account {
                            id: *sender,
                            last_seen: 0,
                        });
                        accounts.insert(Account {
                            id: *recipient,
                            last_seen: 0,
                        });

                        Logger::info("LogData types are unused in this example. (>'')>");
                    }
                    _ => {
                        Logger::info("This type is not handled yet.");
                    }
                }
            }

            // Persist the transaction to the database.
            let tx_entity = Tx {
                block: blck.id,
                timestamp: blck.timestamp,
                id: tx.id,
                value: tx_amount,
                status: tx.status.clone().into(),
                tokens_transferred: Jsonb(
                    serde_json::to_value(tokens_transferred)
                        .unwrap()
                        .to_string(),
                ),
            };

            tx_entity.save();
        }

        // Save all of our accounts
        for account in accounts.iter() {
            account.save();
        }

        // Save all of our contracts
        for contract in contracts.iter() {
            contract.save();
        }
    }
}
```
