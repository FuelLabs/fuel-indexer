# Call

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

- A `Call` receipt is generated whenever a function is called in a Sway contract.
- The `fn_name` field contains the name of the called function from the aforementioned contract.
- [Read more about `Call` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#return-receipt)

You can handle functions that produce a `Call` receipt type by adding a parameter with the type `Call`.

```rust, ignore
fn handle_call(call: Call) {
  // handle the emitted Call receipt
}
```
