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

#[indexer(manifest = "examples/dashboard/manifest.yaml")]
mod dashboard_index {

    fn index_dashboard_data(block_data: BlockData) {
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
                        // thus, we leverage each field to generate it
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
