# `Log`

> A `Log` receipt is generated when calling `log()` on a non-reference types in a Sway contracts - specifically `bool`, `u8`, `u16`, `u32`, and `u64`. The `ra` field includes the value being logged while `rb` may include a non-zero value representing a unique ID for the `log` instance. [Read more about `Log` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#log-receipt).

## Definition

```rust, ignore
use fuel_types::ContractId;
pub struct Log {
    pub contract_id: ContractId,
    pub ra: u64,
    pub rb: u64,
}
```

## Usage

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "my_indexer.manifest.yaml")]
mod my_indexer {
    fn handle_log_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Log { contract_id, .. } => {
                        info!("Found log receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```
