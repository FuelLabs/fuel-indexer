# `Burn`

> A `Burn` receipt is generated whenever an asset is burned in a Sway contract. [Read more about `Burn` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#burn-receipt)

## Definition

```rust, ignore
use fuel_types::{AssetId, ContractId};
pub struct Burn {
    pub sub_id: AssetId,
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}
```

## Usage

You can handle functions that produce a `Burn` receipt type by adding a parameter with the type `Burn`.

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_burn_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Burn { contract_id, .. } => {
                        info!("Found burn receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```
