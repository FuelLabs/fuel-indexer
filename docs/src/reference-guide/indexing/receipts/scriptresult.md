# ScriptResult

```rust,ignore
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}
```

- A `ScriptResult` receipt is generated when a contract call resolves; that is, it's generated as a result of the `RET`, `RETD`, and `RVRT` instructions.
- The `result` field will contain a `0` for success, and a non-zero value otherwise.
- [Read more about `ScriptResult` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#scriptresult-receipt)

You can handle functions that produce a `ScriptResult` receipt type by adding a parameter with the type `abi::ScriptResult`.

```rust, ignore
fn handle_script_result(script_result: abi::ScriptResult) {
  // handle the ScriptResult receipt
}
```
