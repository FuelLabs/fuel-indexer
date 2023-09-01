# `BlockData`

> The `BlockData` struct is how blocks are represented in the Fuel indexer. It contains metadata such as the ID, height, and time, as well as a list of the transactions it contains (represented by `TransactionData`). It also contains the public key hash of the block producer, if present.

## Definition

```rust,ignore
pub struct BlockData {
    pub height: u32,
    pub id: Bytes32,
    pub header: Header,
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub consensus: Consensus,
    pub transactions: Vec<TransactionData>,
}
```

## Usage

```rust,ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "my_indexer.manifest.yaml")]
mod my_indexer {
    fn handle_block(block_data: BlockData) {
        let height = block_data.header.height;
        info!("This block #{height}");
    }
}
```