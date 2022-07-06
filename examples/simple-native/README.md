# simple-native

## Setup


```bash
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

## Usage

#### Run migrations
  - In this example we're using an `indexer` database owned by a `postgres` role without a password

```bash
cd fuel-indexer/

DATABASE_URL="postgres://postgres@127.0.0.1:5432/indexer" diesel migration list --migration-dir=schema/migrations/postgres
```

#### Start fuel node and use small webserver as contract proxy

```bash
cd fuel-indexer/examples/simple-native/programs/counter-rs

RUST_LOG=debug cargo run
```

#### Start the fuel indexer service

```bash
cd fuel-indexer/examples/simple-native/

RUST_LOG=info cargo run -- ./config.yaml
```

#### Send a transaction to the smartcontract via the webserver

```bash
curl -X POST http://127.0.0.1:8080/count | json_pp
```

#### Verify data was posted to the database

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
