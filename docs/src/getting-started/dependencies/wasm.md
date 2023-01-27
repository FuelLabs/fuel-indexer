# WASM

Two additonal cargo components will be required to build your indexers: `wasm-snip` and the `wasm32-unknown-unknown` target.

> As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, you can use `wasm-snip` to remove the errant symbols from the WASM module. An example can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

## `wasm-snip`

To install the `wasm-snip`:

```bash
cargo install wasm-snip
```

## `wasm32` target

To install the `wasm32-unknown-unknown` target via `rustup`:

```bash
rustup target add wasm32-unknown-unknown
```
