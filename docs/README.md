![Fuel Logo](./src/img/fuel.png "Fuel Logo")

# Fuel Indexer

[![build](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/fuel-indexer?label=latest)](https://crates.io/crates/fuel-indexer)
[![docs](https://docs.rs/fuel-indexer/badge.svg)](https://docs.rs/fuel-indexer/)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within the Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

- [Documentation](#documentation)
- [Setup](#setup)
  - [Installing Rust](#installing-rust)
  - [`fuelup`](#fuelup)
  - [Database](#database)
    - [PostgreSQL](#postgresql)
    - [SQLite](#sqlite)
    - [SQLx](#sqlx)
  - [Fuel Node](#fuel-node)
  - [`forc index`](#forc-index)
  - [Creating an Index](#creating-an-index)
    - [Schema](#schema)
    - [Manifest](#manifest)
  - [Compiling an Index](#compiling-an-index)
  - [Deploying an Index](#deploying-an-index)
  - [Querying an Index](#querying-an-index)
- [Building from Source](#building-from-source)
  - [Clone repository](#clone-repository)
  - [Run migrations](#run-migrations)
  - [Start the service](#start-the-service)
  - [Run tests](#run-tests)
- [Quickstart](#quickstart)

## Documentation

Full documentation can be found in the [Fuel indexer book](https://fuellabs.github.io/fuel-indexer).

## Setup

### Installing Rust

The Fuel indexer is built in Rust and also uses `cargo`. The easiest way to get both is to use `rustup`, Rust's toolchain installer. Installing Rust using `rustup` will also install `cargo`.

On Linux and macOS systems, this is done as follows:

```bash
curl https://sh.rustup.rs -sSf | sh
```

It will download a script, and start the installation. If everything goes well, you’ll see this appear:

```text
Rust is installed now. Great!
```

On Windows, download and run [rustup-init.exe](https://win.rustup.rs/). It will start the installation in a console and present the above message on success.

After this, you can use the rustup command to also install beta or nightly channels for Rust and Cargo.

For other installation options and information, visit the [install](https://www.rust-lang.org/tools/install) page of the Rust website.

Alternatively, you can [build Cargo from source.](https://github.com/rust-lang/cargo#compiling-from-source)

### `fuelup`

We recommend that you use the Fuel indexer through [`forc`, the Fuel orchestrator](https://fuellabs.github.io/sway/master/forc/index.html). You can get `forc` (and other Fuel components) by way of [`fuelup`, the Fuel toolchain manager](https://fuellabs.github.io/fuelup/latest). Install `fuelup` by running the following command, which downloads and runs the installation script.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

### Database

At this time, the Fuel indexer requires the use of a database. We currently support two database options: PostgreSQL and SQLite. PostgreSQL is a database solution with a complex feature set and requires a database server. SQLite is an embedded database solution with a simpler set of features and can be setup and moved to different systems.

#### PostgreSQL

> Note: The following explanation is for demonstration purposes only. A production setup should use secure users, permissions, and passwords.

On macOS systems, you can install PostgreSQL through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). Once installed, you can add PostgreSQL to your system by running `brew install postgresql`. You can then start the service through `brew services start postgresql`. You'll need to create a database for your index data, which you can do by running `createdb [DATABASE_NAME]`. You may also need to create the `postgres` role; you can do so by running `createuser -s postgres`.

For Linux-based systems, the installation process is similar. First, you should install PostgreSQL according to your distribution's instructions. Once installed, there should be a new `postgres` user account; you can switch to that account by running `sudo -i -u postgres`. After you have switched accounts, you may need to create a `postgres` database role by running `createuser --interactive`. You will be asked a few questions; the name of the role should be `postgres` and you should elect for the new role to be a superuser. Finally, you can create a database by running `createdb [DATABASE_NAME]`.

In either case, your PostgreSQL database should now be accessible at `postgres://postgres@127.0.0.1:5432/[DATABASE_NAME]`.

##### SQLite

On macOS systems, you can install SQLite through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). Once installed, you can add SQLite to your system by running `brew install sqlite`. You can create a database by running `sqlite3 [DATABASE_FILE_PATH]`.

For Linux-based systems, you should first install SQLite according to the instructions for your distribution. Once installed, you can create a database by running `sqlite3 [DATABASE_FILE_PATH]`.

In either case, your SQLite database is now accessible at `sqlite://[DATABASE_FILE_PATH]`.

##### SQLx

After setting up your database, you should install `sqlx-cli` in order to run migrations for your indexer service; you can do so by running `cargo install sqlx-cli --features postgres,sqlite`. Once installed, you can run the migrations by running the following command after changing `DATABASE_URL` to match your setup. For example:

```sh
DATABASE_URL=sqlite://indexer_database.db bash scripts/run_migrations.bash
```

### Fuel Node

As mentioned before, the Fuel indexer indexes data from the Fuel blockchain; thus, the indexer needs to connect to a Fuel node in order to process, transform, and save blockchain data. Once `fuelup` has been installed, you can run a Fuel node locally by running `fuel-core run`.

### `forc index`

You can find all the available Fuel indexer commands by running `forc index`.

```text
❯ forc index
Fuel Index Orchestrator

USAGE:
    forc-index <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    build     Build an index
    check     Get status checks on all indexer components
    deploy    Deploy an index asset bundle to a remote or locally running indexer server
    help      Print this message or the help of the given subcommand(s)
    init      Create a new indexer project in the current directory
    new       Create a new indexer project in a new directory
    remove    Stop and remove a running index
    start     Start a local indexer service
```

## Building from Source

### Clone repository

```bash
git clone git@github.com:FuelLabs/fuel-indexer.git
```

### Run migrations

```bash
DATABASE_URL=postgres://postgres@localhost bash scripts/run_migrations.bash
```

### Start the service

```bash
cargo run --bin fuel-indexer
```

### Run tests

```txt
cargo test --locked --workspace --all-features --all-targets
```

## Quickstart

A cursory explanation on how to get up and running with an index in 5 minutes

> This Quickstart will assume that you've:
>
> - Read over [Getting Started](./../getting-started/index.md)
> - Have installed all relevant [system](./../getting-started/system-dependencies.md) dependencies
> - Have installed all relevant [application](./../getting-started/application-dependencies.md) dependencies
> - Have already created a Fuel project according to [the recommended project structure](./../getting-started/fuel-indexer-project.md)
> - Have installed the [`forc index`](http://localhost:3000/plugins/forc-index.html) plugin via [`fuelup`](http://localhost:3000/getting-started/application-dependencies/fuelup.html)

## What we'll do

In this Quickstart we're going to write a simple Sway 🌴 smart contract, then
build and deploy and index that saves events from this contract into a database.

- [Writing a contract](#writing-a-contract)
- [Create and deploy an index](#create-and-deploy-an-index)
  - [Initializing an index](#1-initialize-a-new-index-project)
  - [GraphQL data models](#2-graphql-data-models)
  - [Update index manifest](#3-update-index-manifest)
  - [Write your index code](#4-write-your-index-code)
  - [Compile the index](#5-compile-the-index)
  - [Start the index service & deploy your index](#6-start-the-service--deploy-your-index)

## What you'll need

For this Quickstart we'll need a few components that include Docker, Postgres,
and the Fuel Indexer service. In order to see which components you have
installed already, simply use the `forc index check` command, which will check
for these executables in your `$PATH`.

```text
➜  forc index check

❌ Could not connect to indexers service: error sending request for url (http://127.0.0.1:29987/api/health): error trying to connect: tcp connect error: Connection refused (os error 61)

+--------+------------------------+----------------------------------------------------------------------------+
| Status |       Component        |                                  Details                                   |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuel-indexer binary    |  Found 'fuel-indexer' at '/Users/rashad/.fuelup/bin/fuel-indexer'          |
+--------+------------------------+----------------------------------------------------------------------------+
|   ⛔️   | fuel-indexer service   |  Failed to detect a locally running fuel-indexer service at Port(29987).   |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | psql                   |  Found 'psql' at '/usr/local/bin/psql'                                     |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | sqlite                 |  Found 'sqlite' at '/usr/bin/sqlite3'                                      |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuel-core              |  Found 'fuel-core' at '/Users/rashad/.fuelup/bin/fuel-core'                |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | docker                 |  Found 'docker' at '/usr/local/bin/docker'                                 |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuelup                 |  Found 'fuelup' at '/Users/rashad/.fuelup/bin/fuelup'                      |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | wasm-snip              |  Found 'wasm-snip' at '/Users/rashad/.cargo/bin/wasm-snip'                 |
+--------+------------------------+----------------------------------------------------------------------------+
```

----

### Writing a contract

`cd contracts/ && forc new greeting`

Write a "greeting" type of Sway smart contract.

```sway
// src/main.sw
contract;

use std::logging::log;

struct Person {
    name: str[32],
}

struct Greeting {
    id: u64,
    greeting: str[32],
    person: Person,
}

abi Greet {
    fn new_greeting(id: u64, greeting: str[32], person_name: str[32]);
}

impl Greet for Contract {
    fn new_greeting(id: u64, greeting: str[32], person_name: str[32]) {
        log(Greeting{ id, greeting, person: Person{ name: person_name }});
    }
}

```

> Compile your smart contract with [`forc build`](https://fuellabs.github.io/sway/v0.31.2/forc/commands/forc_build.html), which will build the ABI JSON asset required by your index.

## Create and deploy an index

This consists of a few small parts:

1. Initializing a new index project.
2. Defining the structure of your data models and queries via GraphQL schema.
3. Specifying an index _manifest_: a YAML file used to configure your index at compile-time and run-time.
4. Writing the actual code to index events.
5. Compiling your new index code to a WebAssembly binary.
6. Kicking off a local indexer service and deploying your newly created index

> Below we're using the `forc index` plugin provided by [`forc`](https://fuellabs.github.io/sway/v0.31.1/forc/plugins/index.html).

### 1. Initialize a new index project

```bash
➜ mkdir indexer

➜ cd indexer/ && forc index new hello-index

➜ cd hello-index

➜ hello-index tree .
.
├── Cargo.toml
├── hello_index.manifest.yaml
├── schema
│   └── hello_index.schema.graphql
└── src
    └── lib.rs

2 directories, 4 files
```

### 2. GraphQL data models

If you open up `hello-index/schema/hello_index.schema.graphql`

```graphql
schema {
    query: QueryRoot
}

type QueryRoot {
    greeting: Greeting
    greeter: Greeter
}

# Calling this `Greeter` so as to not clash with `Person` in the contract
type Greeter {
    id: ID!
    name: Charfield!
    first_seen: UInt8!
    last_seen: UInt8!
}

# Calling this `Salutation` so as to not clash with `Greeting` in the contract
type Salutation {
    id: ID!
    message_hash: Bytes32!
    message: Charfield!
    greeter: Greeter!
    first_seen: UInt8!
    last_seen: UInt8!
}
```

### 3. Update index manifest

If you open up `hello-index/hello_index.manifest.yaml`

```yaml
namespace: fuel_examples
identifier: hello_index
contract_id: ~
# ABI files are _not_ required. However, in this example, since we already wrote
# and compiled our smart contract, we'll include it's ABI JSON output here.. Note
# that we are using assets located in the examples directory of the fuel-indexer 
# repository
abi: examples/hello-world/contracts/greeting/out/debug/greeting-abi.json
start_block: 1
report_metrics: true
graphql_schema: examples/hello-world/schema/hello_index.schema.graphql
module:
  wasm: ~
```

> Note that we haven't added a `module` parameter to our manifest yet because we haven't actually built a WASM module yet.

### 4. Write your index code

If you open up your index library at `hello-index/src/lib.rs`

```rust
//! A "Hello World" type of program for the Fuel Indexer service.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p hello-index --release --target wasm32-unknown-unknown
//! ```
//!
//! Start a local test Fuel node
//!
//! ```bash
//! cargo run --bin fuel-node
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- --manifest examples/hello-world/hello_index.manifest.yaml
//! ```
//!
//! Now trigger an event.
//!
//! ```bash
//! cargo run --bin hello-bin
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(manifest = "examples/hello-world/hello_index.manifest.yaml")]
mod hello_world_index {

    fn index_logged_greeting(event: Greeting, block: BlockData) {
        // Since all events require a u64 ID field, let's derive an ID using the
        // name of the person in the Greeting
        let greeter_id = first8_bytes_to_u64(&event.person.name.to_string());

        // Here we 'get or create' a Salutation based on the ID of the event
        // emitted in the LogData receipt of our smart contract
        let greeting = match Salutation::load(event.id) {
            Some(mut g) => {
                // If we found an event, let's use block height as a proxy for time
                g.last_seen = block.height;
                g
            }
            None => {
                // If we did not already have this Saluation stored in the database. Here we
                // show how you can use the Charfield type to store strings with length <= 255
                let message =
                    format!("{} 👋, my name is {}", &event.greeting, &event.person.name);

                Salutation {
                    id: event.id,
                    message_hash: first32_bytes_to_bytes32(&message),
                    message,
                    greeter: greeter_id,
                    first_seen: block.height,
                    last_seen: block.height,
                }
            }
        };

        // Here we do the same with Greeter that we did for Saluation -- if we have an event
        // already saved in the database, load it and update it. If we do not have this Greeter
        // in the database then create one
        let greeter = match Greeter::load(greeter_id) {
            Some(mut g) => {
                g.last_seen = block.height;
                g
            }
            None => Greeter {
                id: greeter_id,
                first_seen: block.height,
                name: event.person.name.to_string(),
                last_seen: block.height,
            },
        };

        // Both entity saves will occur in the same transaction
        greeting.save();
        greeter.save();
    }
}
```

### 5. Compile the index

```bash
cd indexer/hello-index

forc index build --manifest hello_index.manifest.yaml
```

> IMPORTANT: As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, a quick workaround requires using `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).
>
>
> IMPORTANT: Be sure to add your new WASM module to your index manifest as shown below.

```yaml
namespace: fuel_examples
identifier: hello_index
abi: examples/hello-world/contracts/greeting/out/debug/greeting-abi.json
start_block: 1
graphql_schema: examples/hello-world/schema/hello_world.schema.graphql
module:
  wasm: target/wasm32-unknown-unknown/release/hello_index.wasm
```

### 6. Start the service & deploy your index

> IMPORTANT: You should already have Postgres running by now.

```bash
# Go back to the repository root
➜ cd fuel-indexer/

# Start a local fuel node
➜  cargo run --bin fuel-node

# Start a local indexer service
➜  forc index start --background 2>/dev/null

# Deploy your index to the local service using test authentication
➜  forc index deploy --manifest hello_index.manifest.yaml
```

If successful, your output should resemble:

```text
➜  forc index deploy --manifest forc_index.manifest.yaml

🚀 Deploying index at hello_index.manifest.yaml to 'http://127.0.0.1:29987/api/index/fuel/hello_index'
{
  "assets": [
    {
      "digest": "d797e33a3d3bbc4d93c7ec09980c220b1243e0ffcd9107b6e13b61cb7704d5ec",
      "id": 1,
      "index_id": 1,
      "version": 1
    },
    {
      "digest": "c5af6d278e29efd47a0493de73509bf4329ca58e47d337d0cc368e0a1f110cb9",
      "id": 1,
      "index_id": 1,
      "version": 1
    },
    {
      "digest": "b32879df38991e7b4f19ed02e394e2d31396cf1fa5ba14429e2af50dfca18cc7",
      "id": 1,
      "index_id": 1,
      "version": 1
    }
  ],
  "success": "true"
}

✅ Successfully deployed index at hello_index.manifest.yaml to http://127.0.0.1:29987/api/index/fuel/hello_index
```

## Generating test data

Now that we've successfully deployed our index, let's make a few calls to our Sway contract in order to produce a few events, and index some data.

```bash

# Go back to the repository root
➜ cd fuel-indexer/

# Run the test data generator for this example
➜ cargo run --bin hello-bin
```

> One contract call will be made, and one event will be emitted to be indexed.
You can continue running this command to generate more data.

----

## Querying for indexed events

After you've successfully completed all six of the aforementioned steps, you can trigger some test events simply by calling the `new_greeting()` method of your Sway contract. This will produce blocks, transactions, and receipts, which will be emitted by your local Fuel node. These events will be picked up by the indexer and subsequently indexed according to the index that you've deployed. Once you have a few indexed events, you can query the indexer for the data that you wish to receive.

### Query for all records of a type

```sh
➜ curl -X POST http://127.0.0.1:29987/api/graph/fuel_examples \
   -H 'content-type: application/json' \
   -d '{"query": "query { greeter { id name first_seen last_seen }}", "params": "b"}' \
| json_pp
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   364  100   287  100    77   6153   1650 --:--:-- --:--:-- --:--:--  9100
[
   {
      "first_seen" : 4,
      "id" : 2314885811371338051,
      "last_seen" : 4,
      "name" : "Ciara"
   },
   {
      "first_seen" : 6,
      "id" : 2314885532299390017,
      "last_seen" : 6,
      "name" : "Alex"
   },
   {
      "first_seen" : 8,
      "id" : 7957705993296504916,
      "last_seen" : 8,
      "name" : "Thompson"
   },
   {
      "first_seen" : 10,
      "id" : 2314885530822735425,
      "last_seen" : 10,
      "name" : "Ava"
   }
]
```