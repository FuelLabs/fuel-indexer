# `MessageOut`

> A `MessageOut` receipt is generated as a result of the `send_typed_message()` Sway method in which a message is sent to a recipient address along with a certain amount of coins.
>
> The `data` field supports data of an arbitrary type `T` and will be decoded by the indexer upon receipt. [Read more about `MessageOut` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#messageout-receipt).

## Definition

```rust,ignore
use fuel_types::{MessageId, Bytes32, Address};
pub struct MessageOut {
    pub message_id: MessageId,
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Bytes32,
    pub len: u64,
    pub digest: Bytes32,
    pub data: Vec<u8>,
}
```

## Usage

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "my_indexer.manifest.yaml")]
mod my_indexer {
    fn handle_message_out(event: MyEvent) {
        info!("Event {event:?} was logged in the contract");
    }
}
```
