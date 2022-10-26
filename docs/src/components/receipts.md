# Receipts

When a Sway contract is deployed and called on a Fuel node, receipts may be generated during its execution. You can think of these receipts as informative objects that are emitted when certain things happen, e.g. transfers, messages, etc. By building [indices](indices/index.md) for receipts, you can create mappings to store this data, allowing your application to answer queries about the contract.

The Fuel indexer currently supports the following receipt types:

- Log
- LogData
- MessageOut
- Transfer
- TransferOut
- ScriptResult

Let's discuss each of the receipts and how you can use them.

## Log

```rust
pub struct Log {
    pub contract_id: ContractId,
    pub ra: u64,
    pub rb: u64,
}
```

A `Log` receipt is generated when calling `log()` in a Sway contract on a non-reference type, specifically `bool`, `u8`, `u16`, `u32`, and `u64`. The `ra` field includes the value being logged while `rb` may include a non-zero value representing a unique ID for the `log` instance. This unique value can be used to determine the data type of the logged value by looking up the ID in the JSON ABI.

## LogData

```rust
pub struct LogData {
    pub contract_id: ContractId,
    pub data: Vec<u8>,
    pub rb: u64,
    pub len: u64,
    pub ptr: u64,
}
```

A `LogData` receipt is generated when calling `log()` in a Sway contract on a reference type; this includes all types _except_ non-reference types. The `data` field will include the logged value as a hexadecimal. Similar to the `Log` receipt, the `rb` field will contain a unique ID that can be used to look up the logged data type.

## MessageOut

```rust
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

A `MessageOut` receipt is generated as a result of the `send_message()` Sway method in which a message is sent to a recipient address along with a certain amount of coins. The `data` field currently supports only a vector of non-reference types rather than something like a struct.

## Transfer

```rust
pub struct Transfer {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}
```

A `Transfer` receipt is generated when coins are transferred to a contract as part of a Sway contract. The `asset_id` field contains the asset ID of the transferred coins, as the FuelVM has built-in support for working with multiple assets. The `pc` and `is` fields aren't currently used for anything, but are included for completeness.

## TransferOut

```rust
pub struct TransferOut {
    pub contract_id: ContractId,
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}
```

A `TransferOut` receipt is generated when coins are transferred to an address rather than a contract. Every other field of the receipt works the same way as it does in the `Transfer` receipt.

## ScriptResult

```rust
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}
```

A `ScriptResult` receipt is generated when a contract call resolves; that is, it's generated as a result of the `RET`, `RETD`, and `RVRT` instructions. The `result` field will contain a `0` for success, and a non-zero value otherwise.
