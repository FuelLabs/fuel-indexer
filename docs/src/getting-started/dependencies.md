# Dependencies

> This guide covers some of the basics with regard to installing dependencies for the Fuel indexer service. However, note that this guide is meant to be a general overview for most platforms and by no means covers all platforms.
>
> If you're having trouble with dependencies on your system, we recommend that you use `docker`.

To run the Fuel indexer, you'll need to install a few dependencies on your system:

1. The [Fuel toolchain](https://docs.fuel.network/guides/installation)
2. A [PostgresQL](#postgresql) server backend
3. The [`wasm32-unknown-unknown`](#web-assembly-wasm) `rustup` target
4. [`wasm-snip`](#web-assembly-wasm), a utility for stripping symbols from WebAssemly binaries.

> If you don't want to install a database directly onto your system, you can use Docker to run a database in an isolated container. You can install Docker by following its [installation instructions](https://docs.docker.com/get-docker/).
>
> For reference purposes, we provide a [`docker compose` file](https://github.com/FuelLabs/fuel-indexer/blob/develop/scripts/docker-compose.yaml) that comes with a PostgresSQL server and a Fuel indexer service.

## The `Fuel` toolchain

Please visit the Fuel's [installation guide](https://docs.fuel.network/guides/installation) to install The Fuel toolchain binaries and pre requisites.

## PostgreSQL

The Fuel indexer requires the use of a database. We currently support [PostgresSQL](https://www.postgresql.org/docs/).

> IMPORTANT: Fuel Indexer users on most platforms don't need to explicitly install PostgresQL software via a package manager. When starting the indexer service via `forc index start` simply pass the `--embedded-database` flag in order to have the indexer service download and start an embedded PostgresQL instance via [`forc index postgres`](../forc-postgres/index.md).
>
> However note that this `--embedded-database` functionality can be a bit brittle or flaky on some platforms, so alternative methods of installing or using PostgresQL are briefly mentioned below.

### macOS

On macOS systems, you can install PostgreSQL through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/).

Once installed, you can add PostgreSQL to your system by running `brew install postgresql`.

## Web Assembly (WASM)

Two additional cargo components will be required to build your indexers: `wasm-snip` and the `wasm32-unknown-unknown` target.

> As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, you can use `wasm-snip` to remove the errant symbols from the WASM module, and prevent this issue from happening. An example can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/develop/scripts/stripper.bash).
>
> Note that since `wasm-snip` strips Web Assembly related symbols, users will temporarily not be allowed to include other WASM-friendly crates (e.g., [`chrono`](https://docs.rs/chrono/latest/chrono/)) in their indexers.

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
