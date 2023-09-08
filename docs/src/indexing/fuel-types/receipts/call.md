# `Call`

> A `Call` receipt is generated whenever a function is called in a Sway contract. The `fn_name` field contains the name of the called function from the aforementioned contract. [Read more about `Call` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#call-receipt).

## Definition

```rust, ignore
use fuel_types::{AssetId, ContractId};
pub struct Call {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub gas: u64,
    pub fn_name: String,
}
```

## Usage

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "my_indexer.manifest.yaml")]
mod my_indexer {
    fn handle_call_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Call { contract_id, .. } => {
                        info!("Found call receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```
