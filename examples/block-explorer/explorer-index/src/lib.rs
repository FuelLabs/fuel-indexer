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
//! Start a local instance of your Fuel Client
//!
//! ```bash
//! fuel-core --db-type in-memory --port 4000 --ip 127.0.0.1
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- --manifest examples/block-explorer/explorer_index.manifest.yaml
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use std::collections::HashSet;

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
    // `BlockData`. A list of native Fuel types can be found at [TODO INSERT LINK]
    fn index_explorer_data(block: BlockData) {
        
        // Here we convert the `BlockData` struct that we get from our Fuel node, into
        // a block entity that we can persist to the database. The `Block` type below is
        // defined in our schema/explorer.graphql and represents the type that we will
        // save to our database.
        let block_entity = Block {
            id: block.height,
            height: block.height,
            timestamp: block.time,
            miner: block.producer,
        };

        // Now that we've created the object for the database, let's save it.
        block_entity.save();

        // Keep track of all accounts involved in this transaction
        let mut accounts = HashSet::new();
        let mut contracts = HashSet::new();

        // Now we'll iterate over all of the transactions in this block, and persist
        // those to the database as well
        for (i, tx) in block.transactions.iter().enumerate() {
            let mut tx_amount = 0;
            let mut tokens_transferred = Vec::new();

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
                    Logger::info("Inside a script transaction.");
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
                    Logger::info("Inside a create transaction.");
                }
            }

            for receipt in &tx.receipts {
                match receipt {
                    #[allow(unused)]
                    Receipt::Call { id, .. } => {
                        contracts.insert(Contract { creator: *id });
                    }
                    #[allow(unused)]
                    Receipt::ReturnData { id, .. } => {
                        contracts.insert(Contract { creator: *id });
                    }
                    #[allow(unused)]
                    Receipt::Transfer {
                        id, to, asset_id, ..
                    } => {
                        contracts.insert(Contract { creator: *id });
                        contracts.insert(Contract { creator: *to });
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
                        tx_amount += amount;
                        accounts.insert(Account { address: *to });
                        tokens_transferred.push(asset_id.to_string());
                    }
                    #[allow(unused)]
                    Receipt::Log { id, .. } => {
                        contracts.insert(Contract { creator: *id });
                    }
                    #[allow(unused)]
                    Receipt::LogData { id, .. } => {
                        contracts.insert(Contract { creator: *id });
                    }
                    #[allow(unused)]
                    Receipt::ScriptResult { result, gas_used } => {}
                    #[allow(unused)]
                    Receipt::MessageOut {
                        sender,
                        recipient,
                        amount,
                        ..
                    } => {
                        tx_amount += amount;
                    }
                    _ => {
                        Logger::info("This type is not handled yet. (>'.')>");
                    }
                }
            }

            let tx_entity = Tx {
                id: i as u64 + block_entity.height,
                block: block_entity.id,
                timestamp: block_entity.timestamp,
                hash: Bytes32::from([0u8; 32]),
                value: 0,
                status: tx.status.clone().into(),
                tokens_transferred: Jsonb(tokens_transferred.join(", ")),
            };

            tx_entity.save();
        }
    }
}
