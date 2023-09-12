
# `LogData`

> A `LogData` receipt is generated when calling `log()` in a Sway contract on a reference type; this includes all types _except_ non-reference types.
>
> The `data` field will include the logged value as a hexadecimal. The `rb` field will contain a unique ID that can be used to look up the logged data type. [Read more about `LogData` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#logdata-receipt).
>
> Note: the example below will run both when the type `MyEvent` is logged as well as when `MyEvent` is returned from a function

## Definition

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

## Usage

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_log_data(event: MyEvent) {
        info!("Event {event:?} was logged in the contract");
    }
}
```
