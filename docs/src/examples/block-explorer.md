# Block Explorer

A rudimentary block explorer backend implementation demonstrating how to leverage basic Fuel indexer abstractions in order to build a cool dApp backend.

```rust
//! A rudimentary block explorer implementation demonstrating how blocks, transactions,
//! contracts, and accounts can be persisted into the database.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p explorer-index --release --target wasm32-unknown-unknown
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
use fuel_indexer_plugin::{types::tx::*, types::Bytes32, utils::sha256_digest};
use std::collections::HashSet;

// Entities require IDs - naively create unique IDs using some caller and the data used
pub fn derive_id(id: [u8; 32], data: Vec<u8>) -> Bytes32 {
    let mut buff: [u8; 32] = [0u8; 32];
    let result = [id.to_vec(), data].concat();
    buff.copy_from_slice(&sha256_digest(&result).as_bytes()[..32]);
    Bytes32::from(buff)
}

// We'll pass our manifest to our #[indexer] attribute. This manifest contains
// all of the relevant configuration parameters in regard to how our index will
// work. In the fuel-indexer repository, we use relative paths (starting from the
// fuel-indexer root) but if you're building an index outside of the fuel-indexer
// project you'll want to use full/absolute paths.
#[indexer(manifest = "examples/block-explorer/explorer_index.manifest.yaml")]
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
        //
        // Note: There is no miner/producer address for blocks in this example; the producer field
        // was removed from the `Block` struct as part of fuel-core v0.12.
        let block = Block {
            id: block_data.id,
            height: block_data.height,
            timestamp: block_data.time,
            gas_limit: block_gas_limit,
        };

        // Now that we've created the object for the database, let's save it.
        block.save();

        // Keep track of some Receipt data involved in this transaction.
        let mut accounts = HashSet::new();
        let mut contracts = HashSet::new();

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
                            id: derive_id(
                                **id,
                                [id.to_vec(), to.to_vec(), asset_id.to_vec()].concat(),
                            ),
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
                            id: derive_id(
                                **id,
                                [id.to_vec(), to.to_vec(), asset_id.to_vec()].concat(),
                            ),
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
                        let log = Log {
                            id: derive_id(**id, u64::to_le_bytes(*rb).to_vec()),
                            contract_id: *id,
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
                            id: derive_id([0u8; 32], u64::to_be_bytes(result).to_vec()),
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

            // Persist the transaction to the database via the `Tx` object defined in the GraphQL schema.
            let tx_entity = Tx {
                block: block.id,
                timestamp: block.timestamp,
                id: tx.id,
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

Once blocks have been added to the database by the indexer, you can query for them by using a query similar to the following:

```sh
curl -X POST http://127.0.0.1:29987/api/graph/fuel_examples \
   -H 'content-type: application/json' \
   -d '{"query": "query { block { id height timestamp }}", "params": "b"}' \
| json_pp
```

```json
[
   {
      "height" : 1,
      "id" : "f169a30cfcbf1eebd97a07b19de98e4b38a4367b03d1819943be41744339d38a",
      "timestamp" : 1668710162
   },
   {
      "height" : 2,
      "id" : "a8c554758f78fe73054405d38099f5ad21a90c05206b5c6137424985c8fd10c7",
      "timestamp" : 1668710163
   },
   {
      "height" : 3,
      "id" : "850ab156ddd9ac9502768f779936710fd3d792e9ea79bc0e4082de96450b5174",
      "timestamp" : 1668710312
   },
   {
      "height" : 4,
      "id" : "19e19807c6988164b916a6877fe049d403d55a07324fa883cb7fa5cdb33438e2",
      "timestamp" : 1668710313
   },
   {
      "height" : 5,
      "id" : "363af43cfd2a6d8af166ee46c15276b24b130fc6a89ce7b3c8737d29d6d0e1bb",
      "timestamp" : 1668710314
   }
]
```
