# Log

```rust, ignore
use fuel_types::ContractId;
pub struct Log {
    pub contract_id: ContractId,
    pub ra: u64,
    pub rb: u64,
}
```

- A `Log` receipt is generated when calling `log()` on a non-reference types in a Sway contracts.
  - Specifically `bool`, `u8`, `u16`, `u32`, and `u64`.
- The `ra` field includes the value being logged while `rb` may include a non-zero value representing a unique ID for the `log` instance.
- [Read more about `Log` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#log-receipt)

You can handle functions that produce a `Log` receipt type by adding a parameter with the type `abi::Log`.

```rust, ignore
fn handle_log(log: abi::Log) {
  // handle the emitted Log receipt
}
```
