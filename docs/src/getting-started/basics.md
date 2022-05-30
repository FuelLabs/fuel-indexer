# Basic usage of the Fuel Indexer

## Instantiating a Fuel client

Start a local Fuel node in the background

```bash
fuel-core --db-type in-memory &
```

## Indexer Setup

Install Deisel CLI if needed

```bash
cargo install diesel_cli --no-default-features --features postgres
```

Run migrations

```bash
cd schema/ && deisel migration run
```

## Instantiating a Fuel Indexer

Start the Fuel Indexer service with a reference to your Fuel node

```bash
RUST_LOG=debug cargo run --bin indexer
```
