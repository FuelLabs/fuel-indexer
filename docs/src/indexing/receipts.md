# Receipts

When a Sway contract is deployed and called on a Fuel node, receipts may be generated during its execution. You can think of these receipts as informative objects that are emitted when certain things happen, e.g. transfers, messages, etc. By building [indices](indices/index.md) for receipts, you can create mappings to store this data, allowing your application to answer queries about the contract.

The Fuel indexer currently supports the following receipt types:

- [Log](https://github.com/FuelLabs/fuel-tx/blob/master/src/receipt.rs#L69)
- [LogData](https://github.com/FuelLabs/fuel-tx/blob/master/src/receipt.rs#L79)
- [MessageOut](https://github.com/FuelLabs/fuel-tx/blob/master/src/receipt.rs#L114)
- [Transfer](https://github.com/FuelLabs/fuel-tx/blob/master/src/receipt.rs#L91)
- [TransferOut](https://github.com/FuelLabs/fuel-tx/blob/master/src/receipt.rs#L100)
- [ScriptResult](https://github.com/FuelLabs/fuel-tx/blob/master/src/receipt.rs#L109)

Below we'll discuss each of these receipts and how you can leverage them to get the most out of your dApp.

## Log

```rust
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
- [Read more about `Log` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#log-receipt)

## LogData

```rust
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
- [Read more about `LogData` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#logdata-receipt)

## MessageOut

```rust
use fuel_types::{MessageId, Bytes32, Address};
pub struct MessageOut {
    pub message_id: MessageId,
    pub sender: Address,
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Bytes32,
    pub len: u64,
    pub digest: Bytes32,
    pub data: Vec<u8>,
}
```

- A `MessageOut` receipt is generated as a result of the `send_message()` Sway method in which a message is sent to a recipient address along with a certain amount of coins.
- The `data` field currently supports only a vector of non-reference types rather than something like a struct.
- [Read more about `MessageOut` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#messageout-receipt)

## Transfer

```rust
use fuel_types::{ContractId, AssetId};
pub struct Transfer {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}
```

- A `Transfer` receipt is generated when coins are transferred to a contract as part of a Sway contract.
- The `asset_id` field contains the asset ID of the transferred coins, as the FuelVM has built-in support for working with multiple assets.
  - The `pc` and `is` fields aren't currently used for anything, but are included for completeness.
- [Read more about `Transfer` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#transfer-receipt)

## TransferOut

```rust
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
- [Read more about `TransferOut` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#transferout-receipt)

## ScriptResult

```rust
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}
```

- A `ScriptResult` receipt is generated when a contract call resolves; that is, it's generated as a result of the `RET`, `RETD`, and `RVRT` instructions.
- The `result` field will contain a `0` for success, and a non-zero value otherwise.
- [Read more about `ScriptResult` in the Fuel protocol ABI spec](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#scriptresult-receipt)
