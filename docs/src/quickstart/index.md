# Quickstart

A cursory explanation on how to get up and running with an index in 5 minutes. This section will assume you've already read over [Getting Started](./../getting-started/index.md) and have already installed all relevant [sytem](./../getting-started/system-dependencies.md) and [application](./../getting-started/application-dependencies.md) dependencies.

## Write a Sway smart contract

`forc new greeting && cd greeting/`

Write a greeting smart contract.

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

> Make sure to compile your smart contract with `forc build`, which will build the ABI JSON asset required by your index.

## Create an index

This consists of 5 parts:

1. Writing your GraphQL schema.
2. Writing your index manifest.
3. Writing the actual code to index events.
4. Compiling your index to WASM.
5. Updating your index manifest with your newly compiled WASM module.

### 1. Start with some GraphQL schema

```graphql
schema {
    query: QueryRoot
}

type QueryRoot {
    greeting: Greeting
    greeter: Greeter
}

type Greeter {
    id: ID!
    name: Bytes32!
    first_seen: UInt8!
    last_seen: UInt8!
}

type Salutation {
    id: ID!
    message_hash: Bytes32!
    message: Jsonb!
    greeter: Greeter!
    first_seen: UInt8!
    last_seen: UInt8!
}


```

### 2. Next write a manifest for your index

```yaml
namespace: fuel_examples
identifier: hello_index
abi: examples/hello-world/contracts/greeting/out/debug/greeting-abi.json
start_block: 1
graphql_schema: examples/hello-world/schema/hello-world.graphql
module:
  wasm: we don't have one of these yet
```

> Note that we haven't added a `module` parameter to our manifest yet because we haven't actually built a WASM module yet.

### 3. Now write the actual code for your index

Start with your Cargo.toml.

```toml
[package]
name = "hello-index"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ['cdylib']

[dependencies]
fuel-indexer-macros = { version = "0.1", path = "../../../fuel-indexer-macros", default-features = false }
fuel-indexer-plugin = { version = "0.1", path = "../../../fuel-indexer-plugin" }
fuel-indexer-schema = { version = "0.1", path = "../../../fuel-indexer-schema", default-features = false }
fuel-tx = "0.23"
fuels-core = "0.30"
fuels-types = "0.30"
getrandom = { version = "0.2", features = ["js"] }
instant = { version = "0.1", default-features = false }
serde = { version = "1.0", default-features = false, features = ["derive"] }
```

Then write your literal indexing code.

```rust
extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::utils::sha256_digest;

// A utility function used to convert an arbitrarily sized string into Bytes32
// using the first 32 bytes of the String
fn bytes32(data: &String) -> Bytes32 {
    let mut buff = [0u8; 32];
    buff.copy_from_slice(&data.as_bytes()[..32]);
    Bytes32::from(buff)
}

// A utility function used to convert an arbitrarily sized string into u64
// using the first 8 bytes of the String
fn u64_id(data: &String) -> u64 {
    let mut buff = [0u8; 8];
    buff.copy_from_slice(&data.as_bytes()[..8]);
    u64::from_le_bytes(buff)
}

#[indexer(manifest = "examples/hello-world/manifest.yaml")]
mod hello_world_index {
    fn index_logged_greeting(event: Greeting, block: BlockData) {
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
                // If we did not already have this Saluation stored in the database, here we
                // show how you can use the Jsonb type to store strings of arbitrary length
                let text =
                    format!("{}, my name is {}", event.greeting, event.person.name);

                Salutation {
                    id: event.id,
                    message_hash: bytes32(&sha256_digest(&text)),
                    message: Jsonb(format!(r#"{{"text":"{text}"}}"#)),
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
                name: bytes32(&event.person.name.to_string()),
                last_seen: block.height,
            },
        };

        // Both entity saves will occur in the same transaction
        greeting.save();
        greeter.save();
    }
}

```

### 4. Compile the index

```bash
cargo build -p hello-index --release --target wasm32-unknown-unknown
```

> IMPORTANT: As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, a quick workaround requires using `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

### 5. Add your new WASM module to your index manifest

```yaml
namespace: fuel_examples
identifier: hello_index
abi: examples/hello-world/contracts/greeting/out/debug/greeting-abi.json
start_block: 1
graphql_schema: examples/hello-world/schema/hello-world.graphql
module:
  wasm: fuel-indexer-tests/assets/hello_index.wasm
```

## Start the indexer

```sh
cargo run --bin fuel-indexer -- --manifest full/path/to/your/manifest.yaml
```

## Query the Indexer

After calling the `count()` method of your Sway contract, query the indexer for the data that you wish to receive.

```sh
curl -X POST http://127.0.0.1:29987/api/graph/fuel_examples \
   -H 'content-type: application/json' \
   -d '{"query": "query { count { id count }}", "params": "b"}' \
| json_pp
```

```json
[
   {
      "id" : 1,
      "count" : "1"
   }
]
```
