# `Mint`

> A `Mint` receipt is generated whenever an asset is burned in a Sway contract. [Read more about `Mint` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#mint-receipt).

## Definition

```rust, ignore
use fuel_types::{AssetId, ContractId};
pub struct Mint {
    pub sub_id: AssetId,
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}
```

## Usage

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "my_indexer.manifest.yaml")]
mod my_indexer {
    fn handle_mint_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Mint { contract_id, .. } => {
                        info!("Found mint receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```
