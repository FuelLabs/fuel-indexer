# Application dependencies

A few application-level dependencies are required to get a Fuel indexer up and running.

## `sqlx-cli`

- More info on [`sqlx-cli`](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)

```bash
cargo install sqlx-cli --features postgres,sqlite
```

- Once `sqlx-cli` is installed, migrations can be run via:

```bash
DATABASE_URL=postgres://postgres@127.0.0.1 bash scripts/run_migrations.bash
```


## `wasm-snip`

> Important: As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, a quick workaround requires using `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

```bash
cargo install wasm-snip
```
