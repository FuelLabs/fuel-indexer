# Counter

> In this example project we'll walk through how to run a Fuel Indexer service that uses WASM-based execution. We'll go through project structure, how to actually use the service, as well as a few key concepts.

## Setup

First we'll walk through the basic setup and usage of the project

```text
➜  tree . -I 'target/'
.
├── README.md
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
├── counter-indexer
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── frontend
│   └── index.html
├── manifest.yaml
├── schema
│   └── counter.graphql
└── web-api-and-fuel-node
    ├── Cargo.toml
    └── src
        └── main.rs

11 directories, 13 files
```

With this [recommended project structure](../getting-started/fuel-indexer-project.md) you'll notice:
- `contracts` is where our Sway smart contract `counter` is located.
- `web-api-and-fuel-node` is a combination of a tiny HTTP web application, and a simple Fuel node, that we use to pass messages to/from the `counter` contract.
- `frontend` is unused (as there is no frontend for this small example)
- `manifest.yaml` is the configuration file for the index that we will be using.

## Usage

In this section we'll cover the exact steps needed to spin up this example.

### Database setup

We'll start by creating a database. This step assumes some familiarity with [creating Postgres roles and databases](https://learn.coderslang.com/0120-databases-roles-and-tables-in-postgresql/). In this example we're using an `indexer` database owned by a `postgres` role without a password.

> Note that some of these commands may differ based on your local setup

```bash
createdb -U postgres indexer
```

Next we'll bootstrap our database by running some migrations. These migrations are responsible for creating our `graph_registry`. 

> The `graph_registry` is a component that is responsible for keeping track of database columns and column types. The `graph_registry` is largely abstracted away from the user, but can be seen by inspecting the `graph_registry` schema.

```bash
DATABASE_URL=postgres://postgres@localhost/indexer bash scripts/run_migrations.local.sh
```

### Starting the web server & Fuel node

As previously mentioned, `web-api-and-fuel-node` contains both a tiny web server used to pass messages to/from our Sway smart contract, as well as our Fuel node that we use to interact with the blockchain. We will start both of these services with the following command.

```bash
cd fuel-indexer/examples/counter/web-api-and-fuel-node && cargo run
```

### Start the Fuel Indexer service

With our Fuel node and web server up and running, we'll next start our Fuel Indexer service.

```bash
cargo build -p fuel-indexer

./target/debug/fuel-indexer --manifest examples/counter/manifest.yaml --fuel-node-port --graphql-api-host 127.0.0.1 --postgres-database indexer
```

### Send a transaction to the smart contract via the web server

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
curl -X POST http://localhost:29987/api/graph/counter -H 'content-type: application/json' -d '{"query": "query { count(id: 1) { id count timestamp } }", "params": "b"}' | json_pp
[
   {
      "count" : 1,
      "id" : 1,
      "timestamp" : 123
   }
]
```

Hooray! 🎉 we've successfully created our first Fuel Indexer service.
