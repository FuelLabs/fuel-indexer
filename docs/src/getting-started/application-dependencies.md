# Application dependencies

We'll need a few application-level dependencies to get our indexer up and running

## `sqlx-cli`

```bash
cargo install sqlx-cli --features postgres,sqlite
```

- More info on [`sqlx-cli`](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)
- We can then run migrations using

```bash
DATABASE_URL=postgres://john:doe@localhost bash scripts/run_migrations.bash
```


## `wasm-snip`

> Important: As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, a quick workaround requires using `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

```bash
cargo install wasm-snip
```
