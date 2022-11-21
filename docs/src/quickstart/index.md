# Quickstart

A cursory explanation on how to get up and running with an index in 5 minutes. This section will assume you've already read over [Getting Started](./../getting-started/index.md) and have already installed all relevant [sytem](./../getting-started/system-dependencies.md) and [application](./../getting-started/application-dependencies.md) dependencies.

## Write a Sway smart contract

`forc new counter && cd counter/`

Write a counter type of smart contract.

```sway
contract;

use std::{address::Address, hash::sha256};

struct Count {
    id: u64,
    count: u64,
}

abi Counter {
    fn count() -> Count;
}

impl Counter for Contract {
    fn count() -> Count {
        Count{ id: 1, count: 1 }
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
    count: CountEntity
}

type CountEntity {
    id: ID!
    count: UInt8!
}

type AdjustedCountEntity {
    id: ID!
    count: CountEntity!
    adjusted_count: UInt8!
}
```

### 2. Next write a manifest for your index

```yaml
namespace: your_org_name
identifier: your_index_name
abi: /full/path/to/your/smart-contract-abi.json
start_block: 1
# your smart contract ID
contract: 0x39150017c9e38e5e280432d546fae345d6ce6d8fe4710162c2e3a95a6faff051 
graphql_schema: /full/path/to/your/graphql.schema
```

> Note that we haven't added a `module` parameter to our manifest yet because we haven't actually built a WASM module yet.

### 3. Now write the actual code for your index

Start with your Cargo.toml.

```toml
[package]
name = "my_index"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ['cdylib']

[dependencies]
fuel-indexer-macros = { version = "0.1", default-features = false }
fuel-indexer-plugin = "0.1"
fuel-indexer-schema = { version = "0.1", default-features = false }
fuel-tx = "0.23"
fuels-core = "0.30"
fuels-types = "0.30"
getrandom = { version = "0.2", features = ["js"] }
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
```

Then write your literal indexing code.

```rust
extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "full/path/to/your/manifest.yaml")]
pub mod my_counter_index_module {

    fn counter_module_handler_one(event: Count) {
        let Count { count, id } = event;

        let count_entity = match CountEntity::load(id) {
            Some(o) => o,
            None => CountEntity { id, count },
        };

        count_entity.save();

        let CountEntity { id, count } = count_entity;

        let adjusted_count_entity = AdjustedCountEntity{ 
            id, 
            count: count_entity.id, 
            adjusted_count: count + 1
        };

        adjusted_count_entity.save();
    }
}

```

### 4. Compile the index

```bash
cargo build -p my-index --release --target wasm32-unknown-unknown
```

> IMPORTANT: As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, a quick workaround requires using `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

### 5. Add your new WASM module to your index manifest

```yaml
namespace: your_org_name
identifier: your_index_name
abi: /full/path/to/your/smart-contract-abi.json
start_block: 1
# your smart contract ID
contract: 0x39150017c9e38e5e280432d546fae345d6ce6d8fe4710162c2e3a95a6faff051 
graphql_schema: /full/path/to/your/graphql.schema
module:
  wasm: /full/path/to/my_index.wasm
```

## Start the indexer

```sh
cargo run --bin fuel-indexer -- --manifest full/path/to/your/manifest.yaml
```

## Query the Indexer

After calling the `count()` method of your Sway contract, query the indexer for the data that you wish to receive.

```sh
curl -X POST http://127.0.0.1:29987/api/graph/your_org_name \
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
