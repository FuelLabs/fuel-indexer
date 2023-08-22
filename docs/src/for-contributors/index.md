# For Contributors

Thanks for your interest in contributing to the Fuel indexer! Below we've compiled a list of sections that you may find useful as you work on a potential contribution:

- [Dependencies](#dependencies)
  - [`fuelup`](#fuelup)
  - [`docker`](#docker)
  - [Database](#database)
    - [PostgreSQL](#postgresql)
  - [SQLx](#sqlx)
- [Building from source](#building-from-source)
- [Run migrations](#run-migrations)
- [Start the service](#start-the-service)
- [Testing](#testing)
  - [Default tests](#default-tests)
  - [End-to-end tests](#end-to-end-tests)
  - [`trybuild` tests](#trybuild-tests)

## Dependencies

### `fuelup`

We use fuelup in order to get the binaries produced by services in the Fuel ecosystem. Fuelup will install binaries related to the Fuel node, the Fuel indexer, the Fuel orchestrator (forc), and other components. `fuelup` can be downloaded [here](https://github.com/FuelLabs/fuelup).

### `docker`

We use Docker to produce reproducible environments for users that may be concerned with installing components with large sets of dependencies (e.g. Postgres). Docker can be downloaded [here](https://docs.docker.com/engine/install/).

### Database

At this time, the Fuel indexer requires the use of a database. We currently support a single database option: Postgres. PostgreSQL is a database solution with a complex feature set and requires a database server.

#### PostgreSQL

> Note: The following explanation is for demonstration purposes only. A production setup should use secure users, permissions, and passwords.

On macOS systems, you can install PostgreSQL through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). Once installed, you can add PostgreSQL to your system by running `brew install postgresql`. You can then start the service through `brew services start postgresql`. You'll need to create a database for your indexed data, which you can do by running `createdb [DATABASE_NAME]`. You may also need to create the `postgres` role; you can do so by running `createuser -s postgres`.

For Linux-based systems, the installation process is similar. First, you should install PostgreSQL according to your distribution's instructions. Once installed, there should be a new `postgres` user account; you can switch to that account by running `sudo -i -u postgres`. After you have switched accounts, you may need to create a `postgres` database role by running `createuser --interactive`. You will be asked a few questions; the name of the role should be `postgres` and you should elect for the new role to be a superuser. Finally, you can create a database by running `createdb [DATABASE_NAME]`.

In either case, your PostgreSQL database should now be accessible at `postgres://postgres@localhost:5432/[DATABASE_NAME]`.

### SQLx

After setting up your database, you should install `sqlx-cli` in order to run migrations for your indexer service. You can do so by running `cargo install sqlx-cli --features postgres`. Once installed, you can run the migrations by running the following command after changing `DATABASE_URL` to match your setup.

## Building from Source

### Clone repository

```bash
git clone git@github.com:FuelLabs/fuel-indexer.git && cd fuel-indexer/
```

### Run migrations

#### Postgres migrations

```sh
cd packages/fuel-indexer-database/postgres
DATABASE_URL=postgres://postgres@localhost sqlx migrate run
```

### Start the service

```bash
cargo run --bin fuel-indexer run
```

You can also start the service with a fresh local node for development purposes:

```bash
cargo run --features fuel-core-lib --bin fuel-indexer run
```

> If no configuration file or other options are passed, the service will default to a `postgres://postgres@localhost` database connection.

## Testing

Fuel indexer tests are currently broken out by a database feature flag. In order to run tests with a Postgres backend, use `--features postgres`.

### Default tests

```bash
cargo test --locked --workspace --all-targets
```

### End-to-end tests

```bash
cargo test --locked --workspace --all-targets --features postgres
```

### `trybuild` tests

For tests related to the meta-programming used in the Fuel indexer, we use `trybuild`.

```bash
RUSTFLAGS='-D warnings' cargo test -p fuel-indexer-macros --locked
```
