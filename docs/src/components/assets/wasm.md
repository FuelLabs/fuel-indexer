# WASM Modules

- WASM modules are compiled binaries that are registered into a Fuel indexer at runtime. The WASM bytes are read in by the indexer and _executors_ are created which will implement blocking calls the to the WASM runtime.

## Usage

In order to compile a WASM module that you've written, you would merely:

```bash
cd /my/wasm/directory && cargo build --release
```

> Important: 
>
> 1. WASM modules are only used if the execution mode specified in your manifest file is `wasm`
>
> 2. As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, a quick workaround requires using `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

