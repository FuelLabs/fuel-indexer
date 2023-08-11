# Burn

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

- A `Burn` receipt is generated whenever an asset is burned in a Sway contract.
- [Read more about `Burn` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#burn-receipt)

You can handle functions that produce a `Burn` receipt type by adding a parameter with the type `Burn`.

```rust, ignore
fn handle_burn(burn: Burn) {
  // handle the emitted Burn receipt
}
```
