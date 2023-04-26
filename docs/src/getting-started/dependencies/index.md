# Dependencies

To run the Fuel indexer, you'll need to install a few dependencies on your system:

- [`fuelup`](./fuelup.md), the Fuel toolchain manager
- A supported [database](./database.md)
  - As of now we only support Postgres
- The [`wasm32-unknown-unknown`](./wasm.md) `rustup` target
- [`wasm-snip`](./wasm.md), a utility for stripping symbols from WebAssemly binaries.

If you don't want to install a database directly onto your system, you can use Docker to run it as an isolated container. You can install it by following the [install instructions](https://docs.docker.com/get-docker/). For reference purposes, we provide a [`docker compose` file](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/docker-compose.yaml) that runs a Postgres database and the Fuel indexer service.

> IMPORTANT: Note for Apple Silicon macOS users: 
>
> Using the Fuel indexer through Docker on Apple Silicon systems is currently not supported. 
>
> We're working to bring support to these systems.

Also, it's assumed that you have the Rust programming language installed on your system. If that is not the case, please refer to the [Rust installation instructions](https://www.rust-lang.org/tools/install) for more information.
