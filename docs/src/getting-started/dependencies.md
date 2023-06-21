# Dependencies

To run the Fuel indexer, you'll need to install a few dependencies on your system:

- [`fuelup`](#fuelup), the Fuel toolchain manager
- A supported [database](#postgresql)
  - As of now we only support Postgres
- The [`wasm32-unknown-unknown`](#wasm) `rustup` target
- [`wasm-snip`](#wasm), a utility for stripping symbols from WebAssemly binaries.

If you don't want to install a database directly onto your system, you can use Docker to run it as an isolated container. You can install it by following the [install instructions](https://docs.docker.com/get-docker/). For reference purposes, we provide a [`docker compose` file](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/docker-compose.yaml) that runs a Postgres database and the Fuel indexer service.

> IMPORTANT: Note for Apple Silicon macOS users: 
>
> Using the Fuel indexer through Docker on Apple Silicon systems is currently not supported. 
>
> We're working to bring support to these systems.

Also, it's assumed that you have the Rust programming language installed on your system. If that is not the case, please refer to the [Rust installation instructions](https://www.rust-lang.org/tools/install) for more information.

## `fuelup`

We strongly recommend that you use the Fuel indexer through [`forc`, the Fuel orchestrator](https://fuellabs.github.io/sway/master/forc/index.html). You can get `forc` (and other Fuel components) by way of [`fuelup`, the Fuel toolchain manager](https://fuellabs.github.io/fuelup/latest). Install `fuelup` by running the following command, which downloads and runs the installation script.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://install.fuel.network/fuelup-init.sh | sh
```

After `fuelup` has been installed, the `forc index` command and `fuel-indexer` binaries will be available on your system.

## PostgreSQL

The Fuel indexer requires the use of a database. We currently support [PostgresSQL](https://www.postgresql.org/docs/).

> IMPORTANT: Fuel Indexer users on most platforms don't need to explicitly install PostgresQL software via a package manager. When starting the indexer service via `forc index start` simply pass the `--embedded-database` flag in order to have the indexer service download and start an embedded PostgresQL instance via [`forc index postgres`](../forc-postgres/index.md).
>
> However if users or devs would like to install PostgresQL via some package manager, feel free to checkout the more detailed installation steps below.

### macOS

On macOS systems, you can install PostgreSQL through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). 

Once installed, you can add PostgreSQL to your system by running `brew install postgresql`. 

## WASM

Two additonal cargo components will be required to build your indexers: `wasm-snip` and the `wasm32-unknown-unknown` target.

> As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, you can use `wasm-snip` to remove the errant symbols from the WASM module. An example can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

### `wasm-snip`

To install the `wasm-snip`:

```bash
cargo install wasm-snip
```

### `wasm32` target

To install the `wasm32-unknown-unknown` target via `rustup`:

```bash
rustup target add wasm32-unknown-unknown
```

> IMPORTANT: Users on Apple Silicon macOS systems may experience trouble when trying to build WASM modules due to its `clang` binary not supporting WASM targets. If encountered, you can install a binary with better support from Homebrew (`brew install llvm`) and instruct `rustc` to leverage it by setting the following environment variables:
>
> - `AR=/opt/homebrew/opt/llvm/bin/llvm-ar`
> - `CC=/opt/homebrew/opt/llvm/bin/clang`
>
> Addtionally, on some systems you need to explictly link clang to llvm. 
>
> - `LIBCLANG_PATH="/opt/homebrew/opt/llvm/lib"`
> - `LDFLAGS="-L/opt/homebrew/opt/llvm/lib"`
> - `CPPFLAGS="-I/opt/homebrew/opt/llvm/include"`
