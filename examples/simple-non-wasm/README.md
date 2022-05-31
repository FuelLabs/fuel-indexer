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

Run migrations

```bash
DATABASE_URL="postgres://postgres@127.0.0.1:5432/indexer" diesel migration list --migration-dir=./schema/migrations
```

Start fuel node and use small webserver as contract proxy

```bash
# OSX
RUST_LOG=debug RUSTFLAGS="-Clink-arg=-Wl" cargo run --target x86_64-apple-darwin
```

Start the fuel indexer service

```bash
RUST_LOG=info cargo run -- ./config/config.yaml
```

Send a transaction to the smartcontract via the webserver

```bash
curl -X POST http://127.0.0.1:8080/count | json_pp
```
