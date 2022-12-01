# WASM Modules

- WebAssembly (WASM) modules are compiled binaries that are registered into a Fuel indexer at runtime. The WASM bytes are read in by the indexer and _executors_ are created which will implement blocking calls the to the WASM runtime.

## Usage

In order to compile a WASM module that you've written, you would merely:

```bash
cd /my/index-lib && cargo build --release
```

## Notes on WASM

There are a few points that Fuel indexer users should know when using WASM:

1. WASM modules are only used if the execution mode specified in your manifest file is `wasm`.

2. Developers should be aware of what things may not work off-the-shelf in a module: file I/O, thread spawning, and anything that depends on system libraries. This is due to the technological limitations of WASM as a whole; more information can be found [here](https://rustwasm.github.io/docs/book/reference/which-crates-work-with-wasm.html).

3. As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due to an errant upstream dependency. For now, a quick workaround requires the use of `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

4. Users on Apple Silicon macOS systems may experience trouble when trying to build WASM modules due to its `clang` binary not supporting WASM targets. If encountered, you can install a binary with better support from Homebrew (`brew install llvm`) and instruct `rustc` to leverage it by setting the following environment variables:

- `AR=/opt/homebrew/opt/llvm/bin/llvm-ar`
- `CC=/opt/homebrew/opt/llvm/bin/clang`
