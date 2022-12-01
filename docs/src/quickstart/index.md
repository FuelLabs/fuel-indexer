# Quickstart

- A cursory explanation on how to get up and running with an index in 5 minutes.
- This will assume that you've:
  - Read over [Getting Started](./../getting-started/index.md)
  - Have installed all relevant [system](./../getting-started/system-dependencies.md) dependencies.
  - Have installed all relevant [application](./../getting-started/application-dependencies.md) dependencies.
  - Have already created a Fuel project according to [the recommended project structure](./../getting-started/fuel-indexer-project.md)

## Write a Sway smart contract

`cd contracts/ && forc new greeting`

Write a greeting smart contract.

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
├── hello-index.manifest.yaml
├── schema
│   └── hello-index.schema.graphql
└── src
    └── lib.rs

2 directories, 4 files
```

### 2. Add some GraphQL type definitions

If you open up `hello-index/schema/hello-index.schema.graphql`

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

### 3. Next update the manifest for your index

If you open up `hello-index/hello-index.manifest.yaml`

```yaml
namespace: fuel_examples
identifier: hello_index
# ABI files are _not_ required. However, in this example, since we already wrote
# and compiled our smart contract, we'll include it's ABI JSON output here.
abi: examples/hello-world/contracts/greeting/out/debug/greeting-abi.json
start_block: 1
graphql_schema: examples/hello-world/schema/hello_world.schema.graphql
module:
  wasm: we don't have one of these yet
```

> Note that we haven't added a `module` parameter to our manifest yet because we haven't actually built a WASM module yet.

### 4. Write the actual code for your index

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
//! With your database backend set up, start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- --manifest examples/hello-world/hello-index.manifest.yaml
//! ```
//!
//! Call the contract
//!
//! ```bash
//! cargo run --bin hello-bin
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::{types::Bytes32, utils::sha256_digest};

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

#[indexer(manifest = "examples/hello-world/hello-index.manifest.yaml")]
mod hello_world_index {
    fn index_logged_greeting(event: Greeting, block: BlockData) {
        // Since all events require a u64 ID field, let's derive an ID using the
        // name of the person in the Greeting
        let greeter_id = u64_id(&event.person.name.to_string());

        // Here we 'get or create' a Salutation based on the ID of the event
        // emiited in the LogData receipt of our smart contract
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
                    message_hash: bytes32(&message),
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

cargo build --release --target wasm32-unknown-unknown
```

> IMPORTANT: As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, a quick workaround requires using `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).
>
>
> IMPORTANT: Be sure to add your new WASM module to your index manifest

```yaml
namespace: fuel_examples
identifier: hello_index
abi: examples/hello-world/contracts/greeting/out/debug/greeting-abi.json
start_block: 1
graphql_schema: examples/hello-world/schema/hello_world.schema.graphql
module:
  wasm: target/wasm32-unknown-unknown/release/hello_index.wasm
```

### 6. Start the indexer & deploy your index

> IMPORTANT: You should already have Postgres running by now

```bash

# Start a local fuel node
➜  cargo run --bin fuel-node

# Start a local indexer service
➜  forc index start --background 2>/dev/null

# Deploy your index to the local service using test authentication
➜  forc index deploy --manifest hello-index.manifest.yaml
```

If successful, your output should resemble:

```text
➜  forc index deploy --manifest forc-index.manifest.yaml

🚀 Deploying index at hello-index.manifest.yaml to 'http://127.0.0.1:29987/api/index/fuel/hello_index'
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

✅ Successfully deployed in at hello-index.manifest.yaml to http://127.0.0.1:29987/api/index/fuel/hello_index
```

## Generating test data

Now that we've successfully deployed our index, let's make a few calls to our Sway contract in order to produce a few events, and index some data.

```bash

# Go back to the repository root
➜ cd fuel-indexer/

➜ cargo run --bin hello-bin
```

> One contract call will be made, and one event will be emitted to be indexed.
You can continue running this command to generate more data.

----

## Querying for indexed events

After you've successfully completed all 6 of the aforementioned steps, you can trigger some test events simply by calling the `new_greeting()` method of your Sway contract. This will produce blocks, transactions, and receipts, which will be emitted by your local Fuel node. These events will be picked up by the indexer and subsequently indexed according to the index that you've deployed. Once you have a few indexed events, you can query the indexer for the data that you wish to receive.

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
