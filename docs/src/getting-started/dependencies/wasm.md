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

> IMPORTANT: Users on Apple Silicon macOS systems may experience trouble when trying to build WASM modules due to its `clang` binary not supporting WASM targets. If encountered, you can install a binary with better support from Homebrew (`brew install llvm`) and instruct `rustc` to leverage it by setting the following environment variables:
>
> - `AR=/opt/homebrew/opt/llvm/bin/llvm-ar`
> - `CC=/opt/homebrew/opt/llvm/bin/clang`