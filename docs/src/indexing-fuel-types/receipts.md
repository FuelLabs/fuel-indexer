# Receipts

Every transaction in the Fuel network contains a list of receipts with information about that transaction, including what contract function was called, logged data, data returned from a function, etc.

There are several types of receipts that can be attached to a transaction and indexed. You can learn more about each of these in the sections below.

- [**Burn**](#burn)
- [**Call**](#call)
- [**Log**](#log)
- [**LogData**](#logdata)
- [**MessageOut**](#messageout)
- [**Mint**](#mint)
- [**Panic**](#panic)
- [**Return**](#return)
- [**ReturnData**](#returndata)
- [**Revert**](#revert)
- [**ScriptResult**](#scriptresult)
- [**Transfer**](#transfer)
- [**TransferOut**](#transferout)

## Burn

A `Burn` receipt is generated whenever an asset is burned in a Sway contract. [Read more about `Burn` in the Fuel protocol ABI spec](https://docs.fuel.network/docs/specs/abi/receipts/#burn-receipt).

```rust, ignore
use fuel_types::{AssetId, ContractId};
pub struct Burn {
    pub sub_id: AssetId,
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}
```

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

mod indexer_mod {
    fn handle_burn_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Burn { contract_id, .. } => {
                        info!("Found burn receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## Call

A `Call` receipt is generated whenever a function is called in a Sway contract. The `fn_name` field contains the name of the called function from the aforementioned contract. [Read more about `Call` in the Fuel protocol ABI spec](https://docs.fuel.network/docs/specs/abi/receipts/#call-receipt).

```rust, ignore
use fuel_types::{AssetId, ContractId};
pub struct Call {
    pub contract_id: ContractId,
    pub to: ContractId,
    pub amount: u64,
    pub asset_id: AssetId,
    pub gas: u64,
    pub fn_name: String,
}
```

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_call_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Call { contract_id, .. } => {
                        info!("Found call receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## Log

A `Log` receipt is generated when calling `log()` on a non-reference types in a Sway contracts - specifically `bool`, `u8`, `u16`, `u32`, and `u64`. The `ra` field includes the value being logged while `rb` may include a non-zero value representing a unique ID for the `log` instance. [Read more about `Log` in the Fuel protocol ABI spec](https://docs.fuel.network/docs/specs/abi/receipts/#log-receipt).

```rust, ignore
use fuel_types::ContractId;
pub struct Log {
    pub contract_id: ContractId,
    pub ra: u64,
    pub rb: u64,
}
```

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_log_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Log { contract_id, .. } => {
                        info!("Found log receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## LogData

A `LogData` receipt is generated when calling `log()` in a Sway contract on a reference type; this includes all types _except_ non-reference types. The `data` field will include the logged value as a hexadecimal. The `rb` field will contain a unique ID that can be used to look up the logged data type. [Read more about `LogData` in the Fuel protocol ABI spec](https://docs.fuel.network/docs/specs/abi/receipts/#logdata-receipt).
>

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

> Note: the example below will run both when the type `MyEvent` is logged as well as when `MyEvent` is returned from a function.

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

## MessageOut

A `MessageOut` receipt is generated as a result of the `send_typed_message()` Sway method in which a message is sent to a recipient address along with a certain amount of coins. The `data` field supports data of an arbitrary type `T` and will be decoded by the indexer upon receipt. [Read more about `MessageOut` in the Fuel protocol ABI spec](https://docs.fuel.network/docs/specs/abi/receipts/#messageout-receipt).

```rust,ignore
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

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_message_out(event: MyEvent) {
        info!("Event {event:?} was logged in the contract");
    }
}
```

## Mint

A `Mint` receipt is generated whenever an asset is burned in a Sway contract. [Read more about `Mint` in the Fuel protocol ABI spec](https://docs.fuel.network/docs/specs/abi/receipts/#mint-receipt).

```rust, ignore
use fuel_types::{AssetId, ContractId};
pub struct Mint {
    pub sub_id: AssetId,
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}
```

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_mint_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Mint { contract_id, .. } => {
                        info!("Found mint receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## Panic

A `Panic` receipt is produced when a Sway smart contract call fails for a reason that doesn't produce a revert. The reason field records the reason for the panic, which is represented by a number between 0 and 255. You can find the mapping between the values and their meanings here in the FuelVM [source code](https://github.com/FuelLabs/fuel-vm/blob/master/fuel-asm/src/panic_reason.rs). [Read more about `Panic` in the Fuel protocol spec](https://docs.fuel.network/docs/specs/abi/receipts/#mint-receipt).

```rust, ignore
use fuel_types::ContractId;
pub struct Panic {
    pub contract_id: ContractId, 
    pub reason: u32, 
}
```

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_panic_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Panic { contract_id, .. } => {
                        info!("Found panic receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## Return

A `Return` receipt is generated when returning a non-reference type in a Sway contract, specifically `bool`, `u8`, `u16`, `u32`, and `u64`. The `val` field includes the value being returned. [Read more about `Return` in the Fuel protocol spec](https://docs.fuel.network/docs/specs/abi/receipts/#return-receipt).

```rust, ignore
use fuel_types::ContractId;
pub struct Return {
    pub contract_id: ContractId,
    pub val: u64,
    pub pc: u64,
    pub is: u64,
}
```

You can handle functions that produce a `Return` receipt type by adding a parameter with the type `Return`.

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_return_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Return { contract_id, .. } => {
                        info!("Found return receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## ReturnData

A `ReturnData` receipt is generated when returning a reference type in a Sway contract; this includes all types _except_ non-reference types. The `data` field will include the returned value as a hexadecimal. [Read more about `ReturnData` in the Fuel protocol ABI spec](https://docs.fuel.network/docs/specs/abi/receipts/#returndata-receipt).

```rust, ignore
use fuel_types::ContractId;
pub struct ReturnData {
    id: ContractId,
    data: Vec<u8>,
}
```

> Note: the example below will run both when the type `MyStruct` is logged as well as when `MyStruct` is returned from a function.

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_return_data(event: MyStruct) {
        info!("MyStruct is: {event:#}");
    }
}
```

## Revert

A `Revert` receipt is produced when a Sway smart contract function call fails. The table below lists possible reasons for the failure and their values. The `error_val` field records these values, enabling your indexer to identify the specific cause of the reversion. [Read more about `Revert` in the Fuel protocol spec](https://docs.fuel.network/docs/specs/abi/receipts/#revert-receipt).

```rust, ignore
use fuel_types::ContractId;
pub struct Revert {
    pub contract_id: ContractId,
    pub error_val: u64,
}
```

| Reason                | Value |
|-----------------------|-------|
| FailedRequire         | 0     |
| FailedTransferToAddress | 1     |
| FailedSendMessage     | 2     |
| FailedAssertEq        | 3     |
| FailedAssert          | 4     |

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_revert_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Revert { contract_id, .. } => {
                        info!("Found return receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## ScriptResult

A `ScriptResult` receipt is generated when a contract call resolves; that is, it's generated as a result of the `RET`, `RETD`, and `RVRT` instructions. The `result` field will contain a `0` for success, and a non-zero value otherwise. [Read more about `ScriptResult` in the Fuel protocol spec](https://docs.fuel.network/docs/specs/abi/receipts/#scriptresult-receipt).

```rust,ignore
pub struct ScriptResult {
    pub result: u64,
    pub gas_used: u64,
}
```

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_script_result_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::ScriptResult { result, .. } => {
                        info!("Result from script: {result:?}");
                    }
                }
            }
        }
    }
}
```

## Transfer

A `Transfer` receipt is generated when coins are transferred to a contract as part of a Sway contract. The `asset_id` field contains the asset ID of the transferred coins, as the FuelVM has built-in support for working with multiple assets. The `pc` and `is` fields aren't currently used for anything, but are included for completeness. [Read more about `Transfer` in the Fuel protocol spec](https://docs.fuel.network/docs/specs/abi/receipts/#transfer-receipt).

```rust,ignore
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

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_transfer_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::Transfer { contract_id, .. } => {
                        info!("Found transfer receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```

## TransferOut

A `TransferOut` receipt is generated when coins are transferred to an address rather than a contract. Every other field of the receipt works the same way as it does in the `Transfer` receipt. [Read more about `TransferOut` in the Fuel protocol spec](https://docs.fuel.network/docs/specs/abi/receipts/#transferout-receipt).

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

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_transfer_out_receipt(block_data: BlockData) {
        let height = block_data.header.height;
        if !block_data.transactions.is_empty() {
            let transaction = block_data.transactions[0];
            for receipt in transaction.receipts {
                match receipt {
                    fuel::Receipt::TransferOut { contract_id, .. } => {
                        info!("Found transfer_out receipt from contract {contract_id:?}");
                    }
                }
            }
        }
    }
}
```
