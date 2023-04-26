# Return

```rust, ignore
use fuel_types::ContractId;
pub struct Return {
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}
```

- A `Return` receipt is generated when returning a non-reference type in a Sway contract.
  - Specifically `bool`, `u8`, `u16`, `u32`, and `u64`.
- The `val` field includes the value being returned.
- [Read more about `Log` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#return-receipt)

You can handle functions that produce a `Return` receipt type by adding a parameter with the type `abi::Return`.

```rust, ignore
fn handle_return(data: abi::Return) {
  // handle the emitted Return receipt
}
```
