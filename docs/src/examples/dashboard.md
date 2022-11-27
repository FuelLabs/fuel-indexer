# Dashboard

```rust
//! A basic dashboard example in which transfer interactions with
//! a smart contract are shown.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p dashboard-index --release
//! ```
//!
//! Use the testing script to start your Fuel node and web API
//!
//! ```bash
//! bash scripts/utils/start_test_components.bash
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- --manifest examples/dashboard/manifest.yaml
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::{types::Bytes32, utils::sha256_digest};

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
#[indexer(manifest = "examples/dashboard/manifest.yaml")]
mod dashboard_index {

    // When specifying args to your handler functions, you can either use types defined
    // in your ABI JSON file, or you can use native Fuel types. These native Fuel types
    // include various `Receipt`s, as well as more comprehensive data, in the form of
    // blocks `BlockData` and transactions `TransactionData`. A list of native Fuel
    // types can be found at:
    //
    //  https://github.com/FuelLabs/fuel-indexer/blob/master/fuel-indexer-schema/src/types/fuel.rs#L28
    fn index_dashboard_data(block_data: BlockData) {

        // In this example, we are leveraging the receipts in
        // order to track the individual transfers.
        let receipts: Vec<Receipt> = block_data
            .transactions
            .into_iter()
            .flat_map(|tx| tx.receipts)
            .collect();

        for r in receipts.iter() {
            match r {
                Receipt::Transfer {
                    id,
                    to,
                    asset_id,
                    amount,
                    pc,
                    is,
                    ..
                } => {
                    let transfer = Transfer {
                        // Currently, custom schema types require an ID;
                        // thus, we use each field to generate it.
                        id: derive_id(
                            **id,
                            [
                                id.to_vec(),
                                to.to_vec(),
                                asset_id.to_vec(),
                                amount.to_be_bytes().to_vec(),
                                pc.to_be_bytes().to_vec(),
                                is.to_be_bytes().to_vec(),
                            ]
                            .concat(),
                        ),
                        contract_id: *id,
                        receiver: *to,
                        amount: *amount,
                        asset_id: *asset_id,
                    };

                    transfer.save();
                }

                Receipt::TransferOut {
                    id,
                    to,
                    asset_id,
                    amount,
                    pc,
                    is,
                    ..
                } => {
                    let transfer_out = TransferOut {
                        id: derive_id(
                            **id,
                            [
                                id.to_vec(),
                                to.to_vec(),
                                asset_id.to_vec(),
                                amount.to_be_bytes().to_vec(),
                                pc.to_be_bytes().to_vec(),
                                is.to_be_bytes().to_vec(),
                            ]
                            .concat(),
                        ),
                        contract_id: *id,
                        receiver: *to,
                        amount: *amount,
                        asset_id: *asset_id,
                    };

                    transfer_out.save();
                }

                #[allow(unused)]
                Receipt::MessageOut {
                    message_id,
                    sender,
                    recipient,
                    amount,
                    nonce,
                    len,
                    digest,
                    data,
                    ..
                } => {
                    let message_out = MessageOut {
                        id: *message_id,
                        sender: *sender,
                        recipient: *recipient,
                        amount: *amount,
                    };

                    message_out.save();
                }
                _ => Logger::info("This type is not handled in this example. (>''<)"),
            }
        }
    }
}
```

You can generate `Transfer`, `TransferOut`, and/or `MessageOut` receipts by transferring assets to a contract or wallet. Once the indexer has added records to the database, you can query for certain types of records by using a query similar to the following:

```sh
curl -X POST http://127.0.0.1:29987/api/graph/fuel_examples \
   -H 'content-type: application/json' \
   -d '{"query": "query { transfer { id contract_id receiver amount asset_id }}", "params": "b"}' \
| json_pp
```

You can expect the response to look similar to this:

```json
[
   {
      "amount" : 8737,
      "asset_id" : "0000000000000000000000000000000000000000000000000000000000000000",
      "contract_id" : "0000000000000000000000000000000000000000000000000000000000000000",
      "id" : "6133306666333063303064386364653163373864313331653061643566396330",
      "receiver" : "ccce87b2adff5d13289de12cdbcc0f05aaaafb6b3f98aed1175c41d88d7bc52e"
   },
   {
      "amount" : 1708,
      "asset_id" : "0404040404040404040404040404040404040404040404040404040404040404",
      "contract_id" : "0000000000000000000000000000000000000000000000000000000000000000",
      "id" : "6564623762666465383039386435343531336630343734396439313932376532",
      "receiver" : "ccce87b2adff5d13289de12cdbcc0f05aaaafb6b3f98aed1175c41d88d7bc52e"
   },
   {
      "amount" : 111,
      "asset_id" : "0303030303030303030303030303030303030303030303030303030303030303",
      "contract_id" : "0000000000000000000000000000000000000000000000000000000000000000",
      "id" : "3661313363346363626431376533353932313563336535356536303963623962",
      "receiver" : "ccce87b2adff5d13289de12cdbcc0f05aaaafb6b3f98aed1175c41d88d7bc52e"
   },
   ...
]
```

You can also create a front-end user interface in order to make it easier for users to consume the information that you're indexing. An example of this can be found the `components/frontend` subdirectory of the `dashboard` example in the Fuel indexer source.
