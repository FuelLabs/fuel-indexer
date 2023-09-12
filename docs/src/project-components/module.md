# Indexer modules

> While the Fuel indexer does support native indexer modules, this section will mostly focus on Web Assembly (WASM) modules.

WebAssembly (WASM) modules are compiled binaries that are registered or deployed to a Fuel indexer at runtime. The WASM bytes are read in by the indexer and _executors_ are created which will essentially pass blocks of on-chain data from the FuelVM to your indexers indefinitely.

The WASM module is generated based on your manifest, schema, and your `lib.rs` file.

## `lib.rs`

You can implement the logic for handling events and saving data to the database in your `lib.rs` file in the `src` folder.

Here, you can define which functions handle different events based on the function parameters. If you add a function parameter of a certain type, the function will handle all blocks, transactions, or transaction receipts that contain a matching type.

We can look at the function below as an example:

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {

    // This `log_the_greeting` function will be called, when we find 
    // a `Greeting` in a block.
    fn log_the_greeting(greeter: Greeting) {
        info!("The greeter is: {greeter:?}");
    }
}
```

> You can learn more about what data can be indexed in the [Indexing](../indexing/index.md) section.

---

## Usage

To compile your indexer code to WASM, you'll first need to install the `wasm32-unknown-unknown` target platform through `rustup`, if you haven't done so already.

```bash
rustup add target wasm32-unknown-unknown
```

After that, you can conveniently use the [`forc index`](./../forc-index/index.md) plugin to manager your indexers. Simply use `forc index build` to build your indexer or checkout the [`forc index build`](./../forc-index/build.md) docs for more options.

> ## Notes on Web Assembly modules
>
> There are a few points that Fuel indexer users should know when using WASM:
>
> 1. WASM modules are only used if the execution mode specified in your manifest file is `wasm`.
> 
> 2. Developers should be aware of what things may not work off-the-shelf in a module: file I/O, thread spawning, and anything that depends on system libraries or makes system calls. This is due to the technological limitations of WASM as a whole; more information can be found [here](https://rustwasm.github.io/docs/book/reference/which-crates-work-with-wasm.html).
>
> 3. As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due to an errant upstream dependency. For now, a quick workaround requires the use of `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/develop/scripts/stripper.bash).
>
> 4. Users on Apple Silicon macOS systems may experience trouble when trying to build WASM modules due to its `clang` binary not supporting WASM targets. If encountered, you can install a binary with better support from Homebrew (`brew install llvm`) and instruct `rustc` to leverage it by setting the following environment variables:
> 
> - `AR=/opt/homebrew/opt/llvm/bin/llvm-ar`
> - `CC=/opt/homebrew/opt/llvm/bin/clang`
