<!--
IMPORTANT: This file uses a combination of markdown and HTML. Unfortunately
doesn't look like mdbook supports anchor links? Given this, anchor links
have been implemented in HTML. `markdown` lint will most certainly complain about it. 
 -->
# Simple Native

- <a href="#header-setup">Setup</a>
- <a href="#header-usage">Usage</a>
- <a href="#header-concepts">Concepts</a>

In this example project we'll walk through how to run a Fuel Indexer service that uses non-WASM execution.

First we'll walk through the basic setup and usage of the project, then we'll discuss the details.

<h2 id="header-usage">Setup</h2>

```bash
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

## <h2 id="header-usage">Usage</h2>

In this section we'll cover the exact steps needed to spin up this example.

### Database setup

We'll start by creating a database. This step assumes some familiarity with [creating Postgres roles and databases](https://learn.coderslang.com/0120-databases-roles-and-tables-in-postgresql/). In this example we're using an `indexer` database owned by a `postgres` role without a password.

- Note that some of these commands may differ based on your local setup

```bash
createdb -U postgres indexer
```

Next we'll bootstrap our database by running some migrations. These migrations are repsonsible for creating our `graph_registry`. 

- The `graph_registry` is a component that is responsible for keeping track of database columns and columns types. The `graph_registry` is largely abstracted away from the user, but can be seen by inspecting the `graph_registry` schema.

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

RUST_LOG=info cargo run -- ./config.yaml ./manifest.yaml
```

### Send a transaction to the smartcontract via the webserver

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

Hooray! 🎉 we've successfully created our first Fuel Indexer service that uses non-WASM execution.

<h2 id="header-concepts">Concepts</h2>

So now that we've walked through how to get up and running with this small example, in this section we'll cover some of the concepts.

### Entities

We'll start with our Sway smart contract. You'll see below that we define a struct that we'll be using in this example. Note that per [the conventions regarding types](./../database/types.md) we're using a `u64` for our struct's identifier.

```sway
struct CountEvent {
    id: u64,
    count: u64,
    timestamp: u64,
}
```

Next we see that we're returning our `CountEvent` struct. This returned entity will be made available in the `data` property of the `ReturnData` receipt.

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

We'll start with the handler.

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

We create a service

```rust
    let mut service = IndexerService::new(config.clone())?;
```

We add our handlers to the service

```rust
service.add_native_indexer(manifest, false, vec![count_handler])?;
```
