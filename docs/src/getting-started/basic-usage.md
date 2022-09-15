# Basic Usage

## Introduction

Before starting, you should make sure that you have installed Rust and set up either a Postgres or SQLite user and/or database. We're also assuming that you've created a Sway contract; for more information, check out [The Sway Book](https://fuellabs.github.io/sway/latest/).

## Installation

To get started, clone the `fuel-indexer` repository:

```sh
git clone https://github.com/FuelLabs/fuel-indexer
```

Then, install the Fuel toolchain manager by running the following command:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

> You can visit the `fuelup` [repository](https://github.com/fuellabs/fuelup) for information about the toolchain manager and the source code for the installation script.

Ensure that the Fuel toolchain has been installed by running the installation command:

```sh
fuelup toolchain install latest
```

Depending on your system, you may need to install additional dependencies, mostly related to compiling various parts of the indexer.

### macOS

```sh
brew update
brew install cmake llvm
```

### Linux

#### Debian-based

```sh
apt update
apt install -y cmake pkg-config git \
    gcc build-essential clang libclang-dev llvm
```

#### Arch Linux

```sh
pacman -Syu --needed --noconfirm cmake \
    gcc pkgconf git clang llvm11 llvm11-libs
```

Lastly, you'll need to install the Diesel CLI; this tool helps to manage your database schema.

```sh
cargo install diesel_cli --no-default-features --features "postgres sqlite"
```

## Set Up the Database

Next, you'll need to set up your database and ensure that it's ready to hold data. Start by navigating to the `fuel-indexer` directory and setting the `DATABASE_URL` environment variable according to your particular setup.

For example, if you're using Postgres, that may look something like this, where `USER` and `PASSWORD` are your previously-created Postgres user and password and `DATABASE` is the name of your database:

```sh
export DATABASE_URL="postgres://[USER]:[PASSWORD]@localhost:5432/[DATABASE]"
```

Alternatively, if you're using SQLite, setting the enviroment variable would look like this, where `DATABASE_FILE_PATH` is the path to your SQLite database:

```sh
export DATABASE_URL="sqlite://[DATABASE_FILE_PATH]"
```

Once the variable is set, run the migrations script; this ensures that the structure of your database matches the format expected by the indexer.

```sh
bash scripts/run_migrations.local.sh
```

## Set Up the Indexer

Let's look at an example. We'll be indexing events from a contract that supports querying and adding to a balance. The indexer will be set up alongside our Sway contract using the following structure:

```sh
.
├── Cargo.toml
├── balance-indexer
│   ├── .cargo
│   │   └── config
│   └── src
│       └── lib.rs
├── contracts
│   └── balance
│       ├── Forc.lock
│       ├── Forc.toml
│       ├── out
│       │   └── debug
│       │       ├── balance-abi.json
│       │       ├── balance-storage_slots.json
│       │       └── balance.bin
│       ├── src
│       │   └── main.sw
│       └── tests
│           └── harness.rs
├── frontend
├── manifest.yaml
└── schema
    └── balance.graphql
```

As you can see, this structure has five main parts:

- `contracts` - contains ABI and other generated files for Sway smart contract
- `frontend` - contains assets for front-end use (not used here)
- `manifest.yaml` - contains configuration for your indexer; you can read more about its structure [here](../components/manifest.md)
- `schema` - contains data structures defining your index; more information can be found in the [Schema](../components/schema.md) page
- `balance-indexer/src` - contains handlers for smart contract events

This is the contract for our example; for more information about Sway contracts, please refer to [The Sway Book](https://fuellabs.github.io/sway/latest/).

```rust
contract;

use std::logging::log;
use std::address::Address;

struct BalanceEvent {
    id: u64,
    timestamp: u64,
    amount: u64,
}

struct AddBalanceEvent {
    id: u64,
    address: b256,
    timestamp: u64,
    amount: u64,
}

abi Balance {
    #[storage(write, read)]
    fn init_balance(value: u64) -> BalanceEvent;

    #[storage(read)]
    fn get_balance() -> BalanceEvent;

    #[storage(write, read)]
    fn increment_balance(sender: b256, amount: u64) -> AddBalanceEvent;
}

storage {
    balance: u64 = 0,
}

impl Balance for Contract {
    #[storage(write, read)]
    fn init_balance(value: u64) -> BalanceEvent {
        storage.balance = value;
        log("Balance initialized");
        
        BalanceEvent {
            id: 1,
            amount: storage.balance,
            timestamp: 1000,
        }
    }

    #[storage(read)]
    fn get_balance() -> BalanceEvent {
        log("Balance retrieved");
        
        BalanceEvent {
            id: 1,
            amount: storage.balance,
            timestamp: 1000,
        }
    }

    #[storage(write, read)]
    fn increment_balance(sender: b256, amount: u64) -> AddBalanceEvent {
        let new_total = storage.balance + amount;
        storage.balance = new_total;

        log("Balanced incremented");

        AddBalanceEvent {
            id: 1,
            address: sender,
            amount: storage.balance,
            timestamp: 1000,
        }
    }
}
```

As you can see, the contract has two event types: `BalanceEvent` and `BalanceAddEvent`. We can then create a GraphQL schema to define how these events should be stored in the indexer. One should note that types used here correspond to their Sway equivalent; see the [Data Types](../components/database/types.md) page for more details.

```txt
schema {
    query: QueryRoot
}

type QueryRoot {
    balance: Balance
}

type Balance {
    id: ID!
    amount: UInt8!
    timestamp: UInt8!
}

type AddBalance {
    id: ID!
    address: Address! @indexed
    timestamp: UInt8!
    amount: UInt8!
}
```

Now that we have event types and their corresponding schema defintions, we can create handlers for each event that we want to index.

```rust
extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "examples/balance/contracts/balance/out/debug/balance-abi.json",
    namespace = "balance",
    identifier = "index1"
    schema = "../schema/balance.graphql"
)]

mod balance {
    fn balance_handler(event: BalanceEvent) {
        let balance = Balance {
            id: event.id,
            timestamp: event.timestamp,
            amount: event.amount,
        };

        balance.save()
    }

    fn add_balance_handler(event: AddBalanceEvent) {
        let add_balance = AddBalance {
            id: event.id,
            address: event.address,
            amount: event.amount,
            timestamp: event.timestamp,
        }

        add_balance.save()
    }
}
```

In this block, we can see that there's an `#[indexer]` macro and a `balance` module. The macro takes the ABI spec of our smart contract and GraphQL schema among other things to generate some helper code to make development easier. Inside of the module, there's a handler for each event as we intend to index both of our contract event types. Each handler takes the contract event type and uses to the event information to instantiate the corresponding schema type. In both cases, the `save()` method is called to place the information into the database.

Finally, we can create the mainfest file to inform the indexer of our assets and how to handle events from the Sway contract.

```yaml
namespace: balance
graphql_schema: examples/balance/schema/balance.graphql
identifier: index1
module:
  wasm: target/wasm32-unknown-unknown/release/balance_indexer.wasm
handlers:
  - event: BalanceEvent
    handler: balance_handler
  - event: AddBalanceEvent
    handler: add_balance_handler
```
