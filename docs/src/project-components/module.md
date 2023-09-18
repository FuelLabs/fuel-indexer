# Indexer modules

Indexer modules are compiled binaries that process data from the Fuel blockchain into entity types defined in your schema so that the data can be stored in a database. The Fuel indexer supports both WebAssembly (WASM) and native binaries; however, we **strongly** recommend using WASM binaries.

This document describes the process of creating an indexer module.

## Creating Handlers

Prior to creating a module for an indexer, both the manifest and schema should be created. At compile time, information will be extracted from both of those assets and combined it with your defined logic to create handlers that save data to storage. Let's look at the following example of a module that will be compiled to WASM:

```rust, ignore
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {

    // This `log_the_greeting` function will be called, when we find
    // a `Greeting` in a block.
    fn log_the_greeting(greeting: Greeting) {
        info!("The greeting is: {greeting:?}");
    }
}
```

## What's going on here?

- The first line imports the [prelude](https://docs.rs/fuel-indexer-utils/latest/fuel_indexer_utils/prelude/index.html) from `fuel_indexer_utils`; this allows you to quickly bootstrap an indexer by using common types and traits. Then, we have a module decorated with the `#[indexer]` macro.
    - This macro processes a manifest at the supplied file path, parses your schema and Sway contract ABI (if supplied), and generates code that is combined with handler functions in order to create a complete indexer module.

- Finally, we have an example handler function. You can define which functions handle different events by using the function parameters. If you add a function parameter of a certain type `T`, the function will be triggered whenever that type is found as part of a block, transaction, or receipt.
    - In this example, let's say that you have a Sway contract with a function that logs a `Greeting` struct. When that function executes as part of a transaction, the logged struct will be included in the data that is processed from the Fuel blockchain. Your indexer module will see the struct and execute `log_the_greeting`.

> You can learn more about what data can be indexed and find example handlers in the [Indexing Fuel Types](../indexing-fuel-types/index.md) and [Indexing Custom Types](../indexing-custom-types/index.md) sections.

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
