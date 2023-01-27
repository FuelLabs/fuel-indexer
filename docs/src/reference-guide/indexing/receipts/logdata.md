
# LogData

```rust,ignore
use fuel_types::ContractId;
pub struct LogData {
    pub contract_id: ContractId,
    pub data: Vec<u8>,
    pub rb: u64,
    pub len: u64,
    pub ptr: u64,
}
```

- A `LogData` receipt is generated when calling `log()` in a Sway contract on a reference type; this includes all types _except_ non-reference types.
- The `data` field will include the logged value as a hexadecimal.
  - The `rb` field will contain a unique ID that can be used to look up the logged data type.
- [Read more about `LogData` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#logdata-receipt)
