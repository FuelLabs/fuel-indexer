# Simple Native

> In this example project we'll walk through how to run a Fuel Indexer service that uses native Rust execution. We'll go through project structure, how to actually use the service, as well as a few key concepts.

## Setup

First we'll walk through the basic setup and usage of the project

```text
➜  cd fuel-indexer/examples/simple-native/

➜  tree . -I 'target/'
.
├── Cargo.toml
├── README.md
├── config.yaml
├── contracts
│   └── counter
│       ├── Forc.lock
│       ├── Forc.toml
│       ├── out
│       │   └── debug
│       │       ├── counter-abi.json
│       │       └── counter.bin
│       └── src
│           └── main.sw
├── counter-rs
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── frontend
│   └── index.html
├── manifest.yaml
├── schema
│   └── counter.graphql
└── src
    └── main.rs

10 directories, 14 files
```

With this [recommended project structure](../getting-started/fuel-indexer-project.md) you'll notice:
- `contracts` is where our Sway smart contract `counter` is located.
- `counter-rs` is a tiny HTTP web application that we use to pass messages to/from the `counter` contract.
- `frontend` is unused (as there is no frontend for this small example)
- `config.yaml` is our node config
- `manifest.yaml` is our indexer config

## Usage

In this section we'll cover the exact steps needed to spin up this example.

### Database setup

We'll start by creating a database. This step assumes some familiarity with [creating Postgres roles and databases](https://learn.coderslang.com/0120-databases-roles-and-tables-in-postgresql/). In this example we're using an `indexer` database owned by a `postgres` role without a password.

> Note that some of these commands may differ based on your local setup

```bash
createdb -U postgres indexer
```

Next we'll bootstrap our database by running some migrations. These migrations are responsible for creating our `graph_registry`. 

> The `graph_registry` is a component that is responsible for keeping track of database columns and columns types. The `graph_registry` is largely abstracted away from the user, but can be seen by inspecting the `graph_registry` schema.

```bash
cd fuel-indexer/

DATABASE_URL="postgres://postgres@127.0.0.1:5432/indexer" \
    diesel migration list --migration-dir=schema/migrations/postgres
```

### Starting the webserver & Fuel node

As previously mentioned, `counter-rs` contains both a tiny webserver used to pass messages to/from our Sway smart contract, as well as our Fuel node that we use to interact with the blockchain. We will start both of these services with the following command.

```bash
cd fuel-indexer/examples/simple-native/counter-rs

RUST_LOG=debug cargo run
```

### Start the fuel indexer service

With our Fuel node and webserver up and running, we'll next start our Fuel Indexer service.

```bash
cd fuel-indexer/examples/simple-native/

RUST_LOG=debug cargo run -- --config ./config.yaml --test-manifest manifest.yaml
```

### Send a transaction to the smart contract via the webserver

```bash
curl -X POST http://127.0.0.1:8080/count | json_pp
```

### Verify data was posted to the database

In this example we just created an entity with `id = 1`

```bash
➜  echo "SELECT max(id) FROM counter.count;" | psql -U postgres -d indexer
 max
-----
   1
(1 row)
```

So that's what we query for

```
curl -X POST http://localhost:29987/graph/counter -H 'content-type: application/json' -d '{"query": "query { count(id: 1) { id count timestamp } }", "params": "b"}' | json_pp
[
   {
      "count" : 1,
      "id" : 1,
      "timestamp" : 123
   }
]
```

Hooray! 🎉 we've successfully created our first Fuel Indexer service that uses native Rust execution.

## Concepts

> So now that we've walked through how to get up and running with this small example, in this section we'll cover some of the concepts.

### Entities

We'll start with our Sway smart contract. You'll see below that we define a struct that we'll be using in this example. Note that per [the conventions regarding types](./../database/types.md) we're using a `u64` for our struct's identifier.

```sway
struct CountEvent {
    id: u64,
    count: u64,
    timestamp: u64,
}
```

Next we see that we're returning our `CountEvent` struct in our `init_counter` method. This returned struct will be made available in the `data` property of the `ReturnData` receipt.

> Read more about receipts [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md).

```sway
abi Counter {
    #[storage(write, read)]
    fn init_counter(value: u64) -> CountEvent;

    #[storage(read)]
    fn get_count() -> AnotherCountEvent;

    #[storage(write, read)]
    fn increment_counter(value: u64) -> CountEvent;
}
```

### Handlers

A handler is basically a function that takes input from a block, and does something with that input. Often times, users will want to save some data in a database in their handler -- i.e., "index" the data. Below is an example of a simple `count_handler` handler function that receives a `Receipt` and saves a `Count` entity into the database.

```rust
fn count_handler(receipt: Receipt) -> Option<IndexerResult<NativeHandlerResult>> {
    match receipt {
        Receipt::ReturnData { data, .. } => {
            // Define which params we expect (using the counter-abi.json as a reference)
            let params = ParamType::Struct(vec![ParamType::U64, ParamType::U64, ParamType::U64]);

            // Decode the data into a Token using these params
            let token = ABIDecoder::decode_single(&params, &data).unwrap();

            // Recover the CountEvent from this token
            let event = CountEvent::from_token(token).unwrap();

            // Using the Count entity from the GraphQL schema
            let count = Count {
                id: event.id,
                timestamp: event.timestamp,
                count: event.count,
            };

            Some(Ok(count.pack()))
        }
        _ => None,
    }
}
```
