# A basic "Hello World" indexer

## Write your contract

We're assuming here you have a sway contract written, and you're ready to start indexing. If not you can check out [The Sway Book](https://fuellabs.github.io/sway/latest/) to get started, then come back!

## Project structure

We'll start with the project structure:

```console
hello-indexer/
├── .cargo
│   └── config
├── Cargo.toml
├── contracts
│   └── hello_indexer.json
├── manifest.yaml
├── README.md
├── schema
│   └── schema.graphql
└── src
    └── lib.rs
```

- The project must compile to WASM, and the way to do that would be to have a `.cargo/config` file with the following contents:

```toml
[build]
target = "wasm32-unknown-unknown"
```

- Cargo.toml - the basic dependencies are:

```toml
[package]
name = "hello-indexer"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ['cdylib']

[dependencies]
fuel-indexer = "0.1"
fuel-indexer-macros = "0.1"
fuel-tx = "0.9"
fuels = "0.13"
fuels-abigen-macro = "0.13"
fuels-core = "0.13"
getrandom = { version = "0.2", features = ["js"] }
serde = { version = "1.0", default-features = false, features = ["derive"] }
```

- Next up is the manifest. This will be configuration for your indexer. For example, what contract addresses do your indexers watch?

```yaml
---
namespace: hello_namespace
graphql_schema: schema.graphql
module:
  wasm:
    hello_indexer.wasm
```

- `namespace` - your graph will live within this namespace in the database, and will be important to remember once you get to querying your data.
- `graphql_schema` - we'll have more to say about this in the next section on data types, but this is a file specifying your indexed data types.
- `module` - the code you'll be writing to transform your data

## Defining your data types

The WASM indexer basically takes two sets of data types, one set coming from your sway contract, and the other defined in the form of a GraphQL schema, specifying how you would like your data to be indexed. The code you'll be writing will just transform the data from your contract types and inserts them into a database.

With that in mind, let's start with a sample `contract.json` ABI spec:

```json
[
    {
        "type":"contract",
        "inputs":[
            {
                "name": "my_event",
                "type": "struct LogEvent",
                "components":
                [
                    {
                        "name":"contract",
                        "type":"b256"
                    },
                    {
                        "name":"event_id",
                        "type":"u64"
                    },
                    {
                        "name":"count",
                        "type":"u64"
                    }
                ]
            }
        ],
        "name":"takes_struct",
        "outputs":[]
    }
]
```

- We have one type here, a `LogEvent` type, with a few attributes. This json will come from your sway contract, and defines the types you want to manipulate in WASM.

As for the data structures defining your index, let's use this example:

```graphql
schema {
    query: QueryRoot
}

type QueryRoot {
    event_counts: EventCount
    data2: SomeOtherData
}

type EventCount {
    id: ID!
    account: Address!
    count: UInt8!
}


type SomeOtherData {
    id: ID!
    hash: Bytes32!
}
```

At a minimum, the schema needs a `schema` definition with the `query` type defined. In this example, they are defined at the top of the file.

Now for the code! Fuel indexers use two proc macros to generate the rust types from these specifications.

```rust,ignore
extern crate alloc;
use fuel_indexer_macros::{graphql_schema, handler};
use fuels_abigen_macro::wasm_abigen;

graphql_schema!("hello_namespace", "schema/schema.graphql");
wasm_abigen!(
    no_name,
    "hello-indexer/contracts/hello_indexer.json"
);

#[handler]
fn function_one(event: LogEvent) {
    Logger::info("Callin' the event handler");
    let LogEvent {
        contract,
        event_id,
        count,
    } = event;

    let mut t1 = match EventCount::load(event_id) {
        Some(t) => t,
        None => EventCount {
            id: event_id,
            account: Address::from(contract),
            count: 0,
        },
    };

    t1.count += count;

    t1.save();
}
```

- `function_one` will take the ABI type `LogEvent`, unpack it into a few variables. It is attempting to load an `EventCount` type from the database with the id `event_id`, if it's not found it will create one. It will increment the `count` attribute either way and then `save` it back to the database.
- `graphql_schema` translates your graphql schema into rust types, so you can have access to them in your handler code.
- `wasm_abigen` will similarly translate the contract ABI types into rust types.
- `handler` is another macro that is required for the indexer to be accessible from the indexer service, it is mostly generates some glue code that bridges the interface between native code and the WASM runtime.

## Build it

With all this in place, we can now build the indexer:

```bash
cargo build --release
```

## Run it

- We've previously described how to bring up the fuel service. The manifest.yaml described here is the one we promised to get to in that section. You may now run that command to bring up your indexer.
- Now, you can send in some test transactions, and the data will be available via the API.

## Querying the data

With some data in our indexer, we can now query the API endpoint, remembering that our namespace defined in the manifest file is `hello_namespace`, some example queries will look like this:

```bash
curl -s localhost:29987/api/graph/hello_namespace -XPOST -H 'content-type: application/json' -d '{"query": "query { event_counts { id count } }", "params": "b"}'
curl -s localhost:29987/api/graph/hello_namespace -XPOST -H 'content-type: application/json' -d '{"query": "query { event_counts { id account count } }", "params": "b"}'
curl -s localhost:29987/api/graph/hello_namespace -XPOST -H 'content-type: application/json' -d '{"query": "query { event_counts { count } }", "params": "b"}'
```

With those queries, the response might look something like:

```json
{
  "count": 7,
  "id": 10
}
```

```json
{
  "account": "0000000000000000000000000000000000000000000000000000000000000000",
  "count": 7,
  "id": 10
}
```

```json
{
  "count": 7
}
```
