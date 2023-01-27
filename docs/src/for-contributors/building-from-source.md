# Building from Source

## Clone repository

```bash
git clone git@github.com:FuelLabs/fuel-indexer.git
```

## Run migrations

```bash
DATABASE_URL=postgres://postgres@localhost bash scripts/run_migrations.bash
```

## Start the service

```bash
cargo run --bin fuel-indexer
```

## Run tests

```txt
cargo test --locked --workspace --all-features --all-targets
```
