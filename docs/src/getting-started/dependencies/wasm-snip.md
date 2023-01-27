# `wasm-snip`

As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, you can use `wasm-snip` to remove the errant symbols from the WASM module. An example can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

To install the `wasm-snip` executable, run

```bash
cargo install wasm-snip
```
