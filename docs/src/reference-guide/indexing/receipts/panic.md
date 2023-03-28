# Panic

```rust, ignore
use fuel_types::ContractId;
use fuel_tx::PanicReason;
pub struct Panic {
    pub contract_id: ContractId, 
    pub reason: PanicReason, 
}
```

- A Panic receipt is produced when a Sway smart contract call 
fails for a reason that doesn't produce a revert. 
- The reason is type PanicReason, which is an enum of `u8` variants, you can see the reason values 
and their corresponding meanings in the FuelVM source code [here](https://github.com/FuelLabs/fuel-vm/blob/master/fuel-asm/src/panic_reason.rs)

- [Read more about `Panic` in the Fuel Protocol spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#panic-receipt)
You can handle functions that could produce a `Panic` receipt by adding a parameter with the type `abi::Panic`.

```rust, ignore
fn handle_panic(panic: abi::Panic) {
  // handle the panic 
}
```

