# Mint

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

- A `Mint` receipt is generated whenever an asset is burned in a Sway contract.
- [Read more about `Mint` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#mint-receipt)

You can handle functions that produce a `Mint` receipt type by adding a parameter with the type `Mint`.

```rust, ignore
fn handle_mint(mint: Mint) {
  // handle the emitted Mint receipt
}
```
