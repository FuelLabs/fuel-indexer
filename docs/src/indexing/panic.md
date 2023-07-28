# Panic

```rust, ignore
use fuel_types::ContractId;
pub struct Panic {
    pub contract_id: ContractId, 
    pub reason: u32, 
}
```

- A `Panic` receipt is produced when a Sway smart contract call fails for a reason that doesn't produce a revert.
- The reason field records the reason for the panic, which is represented by a number between 0 and 255. You can find the mapping between the values and their meanings here in the FuelVM [source code](https://github.com/FuelLabs/fuel-vm/blob/master/fuel-asm/src/panic_reason.rs).
- [Read more about `Panic` in the Fuel Protocol spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#panic-receipt)
- You can handle functions that could produce a `Panic` receipt by adding a parameter with the type `Panic`.

```rust, ignore
fn handle_panic(panic: Panic) {
  // handle the emitted Panic receipt 
}
```
