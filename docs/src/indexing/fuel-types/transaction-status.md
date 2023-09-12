
# `TransactionStatus`

> `TransactionStatus` refers to the status of a `Transaction` in the Fuel network.

## Definition

```rust,ignore
pub enum TransactionStatus {
    Failure {
        block_id: String,
        time: DateTime<Utc>,
        reason: String,
    },
    SqueezedOut {
        reason: String,
    },
    Submitted {
        submitted_at: DateTime<Utc>,
    },
    Success {
        block_id: String,
        time: DateTime<Utc>,
    },
}
```

## Usage 

```rust,ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_transaction(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            match transaction.transaction {
                fuel::Transaction::Script(tx) => match tx.status {
                    fuel::TransactionStatus::Success { block_id, time } => {
                        info!(
                            "Transaction {} in block {} was successful at {}",
                            tx.id, block_id, time
                        );
                    }
                },
                _ => {
                    info!("We don't care about this transaction type");
                }
            }
        }
    }
}
```
