# Building from Source

## Clone repository

```bash
git clone git@github.com:FuelLabs/fuel-indexer.git
```

## Run migrations

```bash
cd packages/fuel-indexer-database/postgres
DATABASE_URL=postgres://postgres@localhost sqlx migrate run
```

## Start the service

```bash
cargo run --bin fuel-indexer
```

## Run tests

```txt
cargo test --locked --workspace --all-features --all-targets
```
