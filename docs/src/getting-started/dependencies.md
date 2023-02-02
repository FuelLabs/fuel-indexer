# Dependencies

To run the Fuel indexer, you'll need to install a few dependencies on your system:

- `fuelup`, the Fuel toolchain manager
- a supported database
  - We support Postgres at this time
- `wasm-snip`, a utility for trimming WebAssembly binaries

If you don't want to install a database directly onto your system, you can use Docker to run it as an isolated container. You can install it by following the [install instructions](https://docs.docker.com/get-docker/). For reference purposes, we provide a [`docker compose` file](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/docker-compose.yaml) that runs a Postgres database and the Fuel indexer service.

Also, it's assumed that you have the Rust programming language installed on your system. If that is not the case, please refer to the [Rust installation instructions](https://www.rust-lang.org/tools/install) for more information.

## `fuelup`

We strongly recommend that you use the Fuel indexer through [`forc`, the Fuel orchestrator](https://fuellabs.github.io/sway/master/forc/index.html). You can get `forc` (and other Fuel components) by way of [`fuelup`, the Fuel toolchain manager](https://fuellabs.github.io/fuelup/latest). Install `fuelup` by running the following command, which downloads and runs the installation script.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

After `fuelup` has been installed, the `forc index` command and `fuel-indexer` binaries will be available on your system.

## Database

At this time, the Fuel indexer requires the use of a database. We currently support a single database option: Postgres. Postgres is a database solution with a complex feature set and requires a database server.

### Postgres

> Note: The following explanation is for demonstration purposes only. A production setup should use secure users, permissions, and passwords.

#### macOS

On macOS systems, you can install Postgres through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). Once installed, you can add Postgres to your system by running `brew install postgresql`. You can then start the service through `brew services start postgresql`. You'll need to create a database for your index data, which you can do by running `createdb [DATABASE_NAME]`. You may also need to create the `postgres` role; you can do so by running `createuser -s postgres`.

#### Linux

For Linux-based systems, the installation process is similar. First, you should install Postgres according to your distribution's instructions. Once installed, there should be a new `postgres` user account; you can switch to that account by running `sudo -i -u postgres`. After you have switched accounts, you may need to create a `postgres` database role by running `createuser --interactive`. You will be asked a few questions; the name of the role should be `postgres` and you should elect for the new role to be a superuser. Finally, you can create a database by running `createdb [DATABASE_NAME]`.

In either case, your Postgres database should now be accessible at `postgres://postgres@127.0.0.1:5432/[DATABASE_NAME]`.

## `wasm-snip`

As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due an errant upstream dependency. For now, you can use `wasm-snip` to remove the errant symbols from the WASM module. An example can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

To install the `wasm-snip` executable, run

```bash
cargo install wasm-snip
```
