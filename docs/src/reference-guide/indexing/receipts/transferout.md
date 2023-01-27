# TransferOut

```rust,ignore
use fuel_types::{ContractId, AssetId, Address};
pub struct TransferOut {
    pub contract_id: ContractId,
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}
```

- A `TransferOut` receipt is generated when coins are transferred to an address rather than a contract.
- Every other field of the receipt works the same way as it does in the `Transfer` receipt.
- [Read more about `TransferOut` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/receipts.md#transferout-receipt)
