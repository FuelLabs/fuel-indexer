# MessageOut

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

- A `MessageOut` receipt is generated as a result of the `send_typed_message()` Sway method in which a message is sent to a recipient address along with a certain amount of coins.
- The `data` field supports data of an arbitrary type `T` and will be decoded by the indexer upon receipt.
- [Read more about `MessageOut` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#messageout-receipt)

You can handle functions that produce a `MessageOut` receipt type by adding a parameter with the type `abi::MessageOut`.

```rust, ignore
fn handle_message_out(message_out: abi::MessageOut) {
  // handle the emitted MessageOut receipt
}
```
