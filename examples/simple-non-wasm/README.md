# simple-non-wasm

## Setup


```bash
➜ tree . -I 'target/'                  
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── config
│   ├── config.yaml
│   └── manifest.yaml
├── programs
│   ├── counter
│   │   ├── Forc.lock
│   │   ├── Forc.toml
│   │   ├── out
│   │   │   └── debug
│   │   │       ├── counter-abi.json
│   │   │       └── counter.bin
│   │   └── src
│   │       └── main.sw
│   └── counter-rs
├── schema
│   └── counter.graphql
└── src
    ├── lib.rs
    └── main.rs

9 directories, 13 files
```

## Usage

#### Run migrations
  - In this example we're using an `indexer` database owned by the `postgres` user without a password

```bash
DATABASE_URL="postgres://postgres@127.0.0.1:5432/indexer" diesel migration list --migration-dir=./schema/migrations
```

#### Start fuel node and use small webserver as contract proxy

OSX

```bash
RUST_LOG=debug RUSTFLAGS="-Clink-arg=-Wl" cargo run --target x86_64-apple-darwin
```

Ubuntu 

```bash
RUST_LOG=debug RUSTFLAGS="-Clink-arg=-Wl,--allow-multiple-definition" cargo run
--target x86_64-unknown-linux-gnu
```

#### Start the fuel indexer service

```bash
RUST_LOG=info cargo run -- ./config.yaml
```

#### Send a transaction to the smartcontract via the webserver

```bash
curl -X POST http://127.0.0.1:8080/count | json_pp
```

#### Verify data was posted to the database

```bash
curl -X POST http://localhost:29987/graph/simple_handler -H 'content-type: application/json' -d '{"query": "query { count(id: 47) { id count } }", "params": "b"}' | json_pp
[
   {
      "count" : "1",
      "id" : 47
   }
]
```
