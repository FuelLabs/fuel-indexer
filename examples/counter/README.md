# counter

A simple app that shows indexer functionality.

## Setup


```bash
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

## Usage

### Run migrations

- In this example we're using the default `postgres` database owned by the default `postgres` role without a password

```bash
createdb indexer -U postgres

DATABASE_URL=postgres://postgres@127.0.0.1 bash scripts/run_migrations.local.sh
```

### Start fuel node and use small webserver as contract proxy

```bash
cd fuel-indexer/examples/counter/web-api-and-fuel-node && cargo run
```

### Start the fuel indexer service

```bash
cargo build -p fuel-indexer

./target/debug/fuel-indexer --manifest examples/counter/manifest.yaml
```

### Send a transaction to the smartcontract via the webserver

```bash
curl -X POST http://127.0.0.1:8080/count | json_pp
```

### Verify data was posted to the database

In this example we just created an entity with `id = 1`

```bash
➜  echo "SELECT max(id) FROM counter.count;" | psql -U postgres -d postgres
 max
-----
   1
(1 row)
```

So that's what we query for

```
curl -X POST http://127.0.0.1:29987/api/graph/counter -H 'content-type: application/json' -d '{"query": "query { count(id: 1) { id count timestamp } }", "params": "b"}' | json_pp
[
   {
      "count" : 1,
      "id" : 1,
      "timestamp" : 123
   }
]
```
