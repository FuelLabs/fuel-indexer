# Transfer

```rust,ignore
use fuel_types::{ContractId, AssetId};
pub struct Transfer {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}
```

- A `Transfer` receipt is generated when coins are transferred to a contract as part of a Sway contract.
- The `asset_id` field contains the asset ID of the transferred coins, as the FuelVM has built-in support for working with multiple assets.
  - The `pc` and `is` fields aren't currently used for anything, but are included for completeness.
- [Read more about `Transfer` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#transfer-receipt)

You can handle functions that produce a `Transfer` receipt type by adding a parameter with the type `abi::Transfer`.

```rust, ignore
fn handle_transfer(transfer: abi::Transfer) {
  // handle the Transfer receipt
}
```
