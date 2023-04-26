# ReturnData

```rust, ignore
use fuel_types::ContractId;
pub struct ReturnData {
    id: ContractId,
    data: Vec<u8>,
}
```

- A `ReturnData` receipt is generated when returning a reference type in a Sway contract; this includes all types _except_ non-reference types.
- The `data` field will include the returned value as a hexadecimal.
- [Read more about `ReturnData` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#returndata-receipt)

You can handle functions that produce a `ReturnData` receipt type by using the returned type as a function parameter.

> Note: the example below will run both when the type `MyStruct` is logged as well as when `MyStruct` is returned from a function.

```rust, ignore
fn handle_return_data(data: MyStruct) {
  // handle the emitted ReturnData receipt
}
```
