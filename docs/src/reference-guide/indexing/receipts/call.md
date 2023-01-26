# Call

```rust, ignore
use fuel_types::ContractId;
pub struct Call {
    id: ContractId,
    param1: u64,
}
```

- A `Call` receipt is generated whenever a function is called in a Sway contract.
- The `param1` field holds the function selector value as a hexadecimal.
- [Read more about `Call` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#return-receipt)
