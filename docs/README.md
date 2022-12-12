![Fuel Logo](./src/img/fuel.png "Fuel Logo")

# Fuel Indexer

[![build](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/fuel-indexer?label=latest)](https://crates.io/crates/fuel-indexer)
[![docs](https://docs.rs/fuel-indexer/badge.svg)](https://docs.rs/fuel-indexer/)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within the Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

- [Documentation](#documentation)
- [Quickstart](#quickstart)
  - [Setup](#setup)
    - [Rust](#rust)
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

## Documentation

Full documentation can be found in the [Fuel indexer book](https://fuellabs.github.io/fuel-indexer).

## Quickstart

Additional information and examples can be found in the [Quickstart](https://fuellabs.github.io/fuel-indexer/master/quickstart/index.html) and [Examples](https://fuellabs.github.io/fuel-indexer/master/examples/index.html) sections of the book.

### Setup

#### Rust

The Fuel indexer is coded in Rust and makes extensive use of its features. You can install the current stable release of Rust by using `rustup`. Installing Rust using `rustup` will also install `cargo`, which you also eventually use. You can install Rust and `rustup` by running the following command, which downloads `rustup` and subsequently installs the most recent version of Rust for your system:

```bash
curl https://sh.rustup.rs -sSf | sh
```

#### `fuelup`

We recommend that you use the Fuel indexer through [`forc`, the Fuel orchestrator](https://fuellabs.github.io/sway/master/forc/index.html). You can get `forc` (and other Fuel components) by way of [`fuelup`, the Fuel toolchain manager](https://fuellabs.github.io/fuelup/latest). Install `fuelup` by running the following command, which downloads and runs the installation script.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

#### Database

At this time, the Fuel indexer requires the use of a database. We currently support two database options: PostgreSQL and SQLite. PostgreSQL is a database solution with a complex feature set and requires a database server. SQLite is an embedded database solution with a simpler set of features and can be setup and moved to different systems.

##### PostgreSQL

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

#### Fuel Node

As mentioned before, the Fuel indexer indexes data from the Fuel blockchain; thus, the indexer needs to connect to a Fuel node in order to process, transform, and save blockchain data. Once `fuelup` has been installed, you can run a Fuel node locally by running `fuel-core run`.

### `forc index`

You can find all the available Fuel indexer commands by running `forc index`.

```text
❯ forc index
forc index 0.1.11
Fuel Index Orchestrator

USAGE:
    forc-index <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    check     Get status checks on all indexer components
    deploy    Deploy an index asset bundle to a remote or locally running indexer server
    help      Print this message or the help of the given subcommand(s)
    init      Create a new indexer project in the current directory
    new       Create a new indexer project in a new directory
    start     Start a local indexer service
    stop      Stop a running index
```

### Creating an Index

A typical project that uses the Fuel indexer has the following directory structure:

```bash
.
├── contracts
└── indexer
```

We'll use a small example Sway contract to demonstrate the functionality of the indexer.

```sway
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

If you want to learn more about Sway and writing contracts, feel free to read the [Sway Book](https://fuellabs.github.io/sway).

> Note: You are not required to write a Sway contract to use the Fuel indexer.

Create a new index by entering your `indexer` directory and running `forc index new [PROJECT_NAME]`. For example, after running `forc index new example-index`, this is the structure of the project directory:

```text
.
├── contracts
└── indexer
    └── example-index
        ├── Cargo.toml
        ├── example_index.manifest.yaml
        ├── schema
        │   └── example_index.schema.graphql
        └── src
            └── lib.rs
```

The new project directory contains the following items:

- a `src` folder containing code for your index
- a schema file containing representations of how data should be stored
- a manifest file detailing the configuration of your index
- `Cargo.toml`

#### Schema

The schema defines how data captured by your index should be stored in the database. In its most basic form, a Fuel indexer GraphQL schema should have a `schema` definition that contains a defined query root. The rest of the implementation is up to you. For the example contract above, the schema may look like this:

```graphql
schema {
    query: QueryRoot
}

type QueryRoot {
    greeting: Salutation
    greeter: Greeter
}

# Calling this `Greeter` so as to not clash with `Person` in the contract
type Greeter {
    id: ID!
    name: Charfield!
}

# Calling this `Salutation` so as to not clash with `Greeting` in the contract
type Salutation {
    id: ID!
    message_hash: Bytes32!
    message: Charfield!
    greeter: Greeter!
}
```

More information about the Fuel indexer GraphQL schema can be found in the [Schema](https://fuellabs.github.io/fuel-indexer/master/components/assets/schema.html) section of the book.

#### Manifest

Here's an example of a manifest file:

```yaml
namespace: fuel_examples
identifier: example_index
abi: /path/to/your/contract-abi.json
contract_id: "0x39150017c9e38e5e280432d546fae345d6ce6d8fe4710162c2e3a95a6faff051"
start_block: 1
graphql_schema: /path/to/your/example_index.schema.graphql
module:
  # See "Compiling an Index" below...
  wasm: /path/to/your/example_index.wasm
report_metrics: true
```

- `namespace`: a unique identifier to organize your indices
- `identifier`: a unique identifier to identify your index inside of a namespace
- `abi`: (optional) path to Sway contract JSON ABI generated by `forc build`
- `contract_id`: (optional) ID of the particular contract to which the index should subscribe
- `start_block`: (optional) the start block after which your index should start indexing data
- `graphql_schema`: path to schema containing the data structures that will be saved to your database
- `module`: path to the code that will be run as an executor inside of the indexer
- `report_metrics`: whether to report Promethesus metrics to a Fuel backend

### Compiling an Index

You'll need to write a handler to index outputs from your contract. For this example, we'll use the following code:

```rust
extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

// A utility function used to convert an arbitrarily sized string into Bytes32
// using the first 32 bytes of the String. This might be provided by a standard-ish
// library in the future.
fn bytes32(data: &str) -> Bytes32 {
    let data = sha256_digest(&data);
    let mut buff = [0u8; 32];
    buff.copy_from_slice(&data.as_bytes()[..32]);
    Bytes32::from(buff)
}

// A utility function used to convert an arbitrarily sized string into u64
// using the first 8 bytes of the String. This might be provided by a standard-ish
// library in the future.
fn u64_id(data: &str) -> u64 {
    let mut buff = [0u8; 8];
    buff.copy_from_slice(&data.as_bytes()[..8]);
    u64::from_le_bytes(buff)
}

#[indexer(manifest = "path/to/your/example_index.manifest.yaml")]
mod hello_world_index {
    fn index_logged_greeting(event: Greeting, block: BlockData) {
        // Since all events require a u64 ID field, let's derive an ID using the
        // name of the person in the Greeting
        let greeter_id = u64_id(&event.person.name.to_string());
        let greeter = Greeter {
                id: greeter_id,
                first_seen: block.height,
                name: event.person.name.to_string(),
                last_seen: block.height,
            },

        let message =
            format!("{} 👋, my name is {}", &event.greeting, &event.person.name);

        let greeting = Salutation {
            id: event.id,
            message_hash: bytes32(&message),
            message,
            greeter: greeter_id,
        }

        // Both entity saves will occur in the same transaction
        greeting.save();
        greeter.save();
    }
}

```

For more information on indexing data from a Fuel node, read the [Indexing](https://fuellabs.github.io/fuel-indexer/master/indexing/index.html) section of the book.

Currently, the Fuel indexer uses WebAssembly (WASM) modules to capture data from the Fuel blockchain. You can compile the code for your index by running `cargo build` in your index's directory.

```bash
cargo build --release
```

The WASM module for your index can be found in the `target/wasm32-unknown-unknown/release` subdirectory. For ease of use, feel free to copy the file to a different path on your system. In any case, return to your manifest and fill the `wasm` field of the `module` block with the path to your index's WASM module.

For more information on WASM (including potential issues and workarounds), read the [WASM](https://fuellabs.github.io/fuel-indexer/master/components/assets/wasm.html) section of the book.

### Deploying an Index

You're now ready to deploy an index to the Fuel indexer. Before doing so, make sure that the indexer is configured to connect to a running Fuel node. By default, the indexer is configured to connect to a local node on port 4000.

For more information on how to configure the indexer, feel free to read the [Configuration](https://fuellabs.github.io/fuel-indexer/master/getting-started/configuration.html) section of the Fuel indexer book.

Start the indexer service in the background by running `forc index start --background`. After the service has started, deploy your index using the corresponding manifest file. If successful, the service will respond with hashes for each of the uploaded assets.

```bash
❯ forc index deploy --manifest example_index.manifest.yaml

🚀 Deploying index at example_index.manifest.yaml to 'http://127.0.0.1:29987/api/index/fuel_examples/example_index'
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

✅ Successfully deployed index at example_index.manifest.yaml to http://127.0.0.1:29987/api/index/fuel_examples/example_index
```

### Querying an Index

As you and others interact with your contract, various outputs will be emitted by the Fuel node; the indexer will receive and index them according to your index's configuration. You can query for the data structures defined in your schema by interacting with the Fuel indexer API; to query for the desired data, you can send a `POST` request to the `/api/graph/[NAMESPACE]` route where `[NAMESPACE]` is the namespace specified in your index manifest.

For our example, the IDs and names of greeters can be found using the following query:

```bash
➜ curl -X POST http://127.0.0.1:29987/api/graph/fuel_examples \
   -H 'content-type: application/json' \
   -d '{"query": "query { greeter { id name }}", "params": "b"}' \
| json_pp
```

The indexer will respond with the query's results.

```json
[
   {
      "id" : 2314885811371338051,
      "name" : "Ciara"
   },
   {
      "id" : 2314885532299390017,
      "name" : "Alex"
   },
   {
      "id" : 7957705993296504916,
      "name" : "Thompson"
   },
   {
      "id" : 2314885530822735425,
      "name" : "Ava"
   }
]
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
