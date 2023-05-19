//! A rudimentary block explorer implementation demonstrating how blocks, transactions,
//! contracts, and accounts can be persisted into the database.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p explorer-indexer --release --target wasm32-unknown-unknown
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- run --manifest examples/block-explorer/explorer-indexer/explorer_indexer.manifest.yaml
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

// We'll pass our manifest to our #[indexer] attribute. This manifest contains
// all of the relevant configuration parameters in regard to how our index will
// work. In the fuel-indexer repository, we use relative paths (starting from the
// fuel-indexer root) but if you're building an index outside of the fuel-indexer
// project you'll want to use full/absolute paths.
#[indexer(
    manifest = "examples/block-explorer/explorer-indexer/explorer_indexer.manifest.yaml"
)]
mod explorer_index {
    // When specifying args to your handler functions, you can either use types defined
    // in your ABI JSON file, or you can use native Fuel types. These native Fuel types
    // include various `Receipt`s, as well as more comprehensive data, in the form of
    // blocks `BlockData` and transactions `TransactionData`. A list of native Fuel
    // types can be found at:
    //
    //  https://github.com/FuelLabs/fuel-indexer/blob/master/fuel-indexer-schema/src/types/fuel.rs#L28
    fn index_explorer_data(block_data: BlockData) {
        let mut block_gas_limit = 0;

        // Convert the deserialized block `BlockData` struct that we get from our Fuel node, into
        // a block entity `Block` that we can persist to the database. The `Block` type below is
        // defined in our schema/explorer.graphql and represents the type that we will
        // save to our database.
        let producer = block_data.producer.unwrap_or(Bytes32::zeroed());

        let block = Block {
            id: first8_bytes_to_u64(block_data.id),
            height: block_data.height,
            producer,
            hash: block_data.id,
            timestamp: block_data.time,
            gas_limit: block_gas_limit,
        };

        // Now that we've created the object for the database, let's save it.
        block.save();

        for tx in block_data.transactions.iter() {
            let mut tx_amount = 0;
            let mut tokens_transferred = Vec::new();

            // `Transaction::Script`, `Transaction::Create`, and `Transaction::Mint`
            // are unused but demonstrate properties like gas, inputs,
            // outputs, script_data, and other pieces of metadata. You can access
            // properties that have the corresponding transaction `Field` traits
            // implemented; examples below.
            match &tx.transaction {
                #[allow(unused)]
                Transaction::Script(t) => {
                    Logger::info("Inside a script transaction. (>^‿^)>");

                    let gas_limit = t.gas_limit();
                    let gas_price = t.gas_price();
                    let maturity = t.maturity();
                    let script = t.script();
                    let script_data = t.script_data();
                    let receipts_root = t.receipts_root();
                    let inputs = t.inputs();
                    let outputs = t.outputs();
                    let witnesses = t.witnesses();

                    let json = &tx.transaction.to_json();
                    block_gas_limit += gas_limit;
                }
                #[allow(unused)]
                Transaction::Create(t) => {
                    Logger::info("Inside a create transaction. <(^.^)>");

                    let gas_limit = t.gas_limit();
                    let gas_price = t.gas_price();
                    let maturity = t.maturity();
                    let salt = t.salt();
                    let bytecode_length = t.bytecode_length();
                    let bytecode_witness_index = t.bytecode_witness_index();
                    let inputs = t.inputs();
                    let outputs = t.outputs();
                    let witnesses = t.witnesses();
                    let storage_slots = t.storage_slots();
                    block_gas_limit += gas_limit;
                }
                #[allow(unused)]
                Transaction::Mint(t) => {
                    Logger::info("Inside a mint transaction. <(^‿^<)");

                    let tx_pointer = t.tx_pointer();
                    let outputs = t.outputs();
                }
            }

            for receipt in &tx.receipts {
                // You can handle each receipt in a transaction `TransactionData` as you like.
                //
                // Below demonstrates how you can use parts of a receipt `Receipt` in order
                // to persist entities defined in your GraphQL schema, to the database.
                match receipt {
                    #[allow(unused)]
                    Receipt::Call { id, .. } => {
                        let contract = Contract {
                            id: first8_bytes_to_u64(bytes32_from_inputs(
                                id,
                                [id.to_vec()].concat(),
                            )),
                            contract_id: *id,
                            last_seen: 0,
                        };

                        contract.save();
                    }
                    #[allow(unused)]
                    Receipt::ReturnData { id, .. } => {
                        let contract = Contract {
                            id: first8_bytes_to_u64(bytes32_from_inputs(
                                id,
                                [id.to_vec()].concat(),
                            )),
                            contract_id: *id,
                            last_seen: 0,
                        };
                        contract.save();
                    }
                    #[allow(unused)]
                    Receipt::Transfer {
                        id,
                        to,
                        asset_id,
                        amount,
                        ..
                    } => {
                        let contract = Contract {
                            id: first8_bytes_to_u64(bytes32_from_inputs(
                                id,
                                [id.to_vec()].concat(),
                            )),
                            contract_id: *id,
                            last_seen: 0,
                        };

                        contract.save();

                        let transfer = Transfer {
                            id: first8_bytes_to_u64(bytes32_from_inputs(
                                id,
                                [id.to_vec(), to.to_vec(), asset_id.to_vec()].concat(),
                            )),
                            contract_id: contract.id,
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
                        let account = Account {
                            id: 1,
                            address: *to,
                            last_seen: 0,
                        };

                        account.save();

                        let contract = Contract {
                            id: 1,
                            contract_id: *id,
                            last_seen: 0,
                        };
                        contract.save();

                        tx_amount += amount;
                        let transfer_out = TransferOut {
                            id: first8_bytes_to_u64(bytes32_from_inputs(
                                id,
                                [id.to_vec(), to.to_vec(), asset_id.to_vec()].concat(),
                            )),
                            contract_id: contract.id,
                            receiver: account.id,
                            amount: *amount,
                            asset_id: *asset_id,
                        };

                        transfer_out.save();
                    }
                    #[allow(unused)]
                    Receipt::Log { id, rb, .. } => {
                        let log = Log {
                            id: first8_bytes_to_u64(bytes32_from_inputs(
                                id,
                                u64::to_le_bytes(*rb).to_vec(),
                            )),
                            contract_id: *id,
                            rb: *rb,
                        };

                        log.save();
                    }
                    #[allow(unused)]
                    Receipt::LogData { id, .. } => {
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
                            id: first8_bytes_to_u64(bytes32_from_inputs(
                                &[0u8; 32],
                                u64::to_be_bytes(result).to_vec(),
                            )),
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

                        let account = Account {
                            id: 1,
                            address: *sender,
                            last_seen: 0,
                        };

                        account.save();

                        Logger::info("LogData types are unused in this example. (>'')>");
                    }
                    _ => {
                        Logger::info("This type is not handled yet.");
                    }
                }
            }

            // Persist the transaction to the database via the `Tx` object defined in the GraphQL schema.
            let tx_entity = Tx {
                block: block.id,
                hash: tx.id,
                timestamp: block.timestamp,
                id: first8_bytes_to_u64(tx.id),
                value: tx_amount,
                status: tx.status.clone().into(),
                tokens_transferred: Json(
                    serde_json::to_value(tokens_transferred)
                        .unwrap()
                        .to_string(),
                ),
            };

            tx_entity.save();
        }
    }
}
