# Revert

```rust, ignore
use fuel_types::ContractId;
pub struct Revert {
    pub contract_id: ContractId,
    pub error_val: u64,
  }
```

- A `Revert` receipt is produced when a Sway smart contract function call fails. 
- The table below lists possible reasons for the failure and their values. 
- The `error_val` field records these values, enabling your indexer to identify the specific cause of the reversion.

| Reason                | Value |
|-----------------------|-------|
| FailedRequire         | 0     |
| FailedTransferToAddress | 1     |
| FailedSendMessage     | 2     |
| FailedAssertEq        | 3     |
| FailedAssert          | 4     |

- [Read more about `Revert` in the Fuel Protocol spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#revert-receipt)

You can handle functions that could produce a `Revert` receipt by adding a parameter with the type `abi::Revert`.

```rust, ignore
fn handle_revert(revert: abi::Revert) {
  // handle the revert 
}
```
