# 🗃 Fuel Indexer
<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD025 -->
![Fuel Logo](./src/img/fuel.png "Fuel Logo")

[![build](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/fuel-indexer?label=latest)](https://crates.io/crates/fuel-indexer)
[![docs](https://docs.rs/fuel-indexer/badge.svg)](https://docs.rs/fuel-indexer/)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within the Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

<!-- Using an <img> so we can size it -->
<img src="https://i.imgur.com/8K14p9h.png" alt="diagram" width="500"/>

- [**For Users**](#for-users)
  - [Dependencies](#dependencies)
    - [`fuelup`](#fuelup)
    - [`wasm`](#wasm)
  - [Quickstart](#quickstart)
  - [`forc index` Plugin](#forc-index-plugin)
    - [`check`](#forc-index-check)
    - [`new`](#forc-index-new)
    - [`init`](#forc-index-init)
    - [`build`](#forc-index-build)
    - [`deploy`](#forc-index-new)
    - [`remove`](#forc-index-remove)
  - [Schema](#schema)
  - [Modules](#modules)
    - [WASM module notes](#notes-on-wasm-modules)
- [**For Contributors**](#for-contributors)
  - [Dependencies](#dev-dependencies)
    - [`fuelup`](#fuelup)
    - [`docker`](#docker)
    - [PostgreSQL](#postgresql)
    - [SQLx](#sqlx)
  - [Building from source](#building-from-source)
  - [Run migrations](#run-migrations)
  - [Start the service](#start-the-service)
  - [Testing](#testing)
    - [Default tests](#default-tests)
    - [End-to-end tests](#end-to-end-tests)
    - [`trybuild` tests](#trybuild-tests)
  - [Contributing](#contributing)
- [Read the book](#read-the-book)

# For Users

Users of the Fuel indexer project include dApp developers looking to write flexible data-based backends for their dApp frontends, as well as index operators who are interested in managing one or many indexer projects for dApp developers.

## Dependencies

### `fuelup`

- We use fuelup in order to get the binaries produced by services in the Fuel ecosystem. Fuelup will install binaries related to the Fuel node, the Fuel indexer, the Fuel orchestrator (forc), and other components.
- fuelup can be downloaded [here](https://github.com/FuelLabs/fuelup).

### `wasm`

Two additonal cargo components will be required to build your indexers: `wasm-snip` and the `wasm32-unknown-unknown` target.

- To install `wasm-snip`:

```bash
cargo install wasm-snip
```

To install the `wasm32-unknown-unknown` target via `rustup`:

```bash
rustup target add wasm32-unknown-unknown
```

> IMPORTANT: Users on Apple Silicon macOS systems may experience trouble when trying to build WASM modules due to its `clang` binary not supporting WASM targets. If encountered, you can install a binary with better support from Homebrew (`brew install llvm`) and instruct `rustc` to leverage it by setting the following environment variables:
>
> - `AR=/opt/homebrew/opt/llvm/bin/llvm-ar`
> - `CC=/opt/homebrew/opt/llvm/bin/clang`

## Quickstart

In this tutorial you will:

1. Bootstrap your development environment.
2. Create, build, and deploy an index to an indexer service hooked up to Fuel's `beta-2` testnet.
3. Query the indexer service for indexed data using GraphQL.

## 1. Setting up your environment

In this Quickstart, we'll use Docker's Compose to spin up a Fuel indexer service with a PostgreSQL database backend. We will also use Fuel's toolchain manager [`fuelup`](https://github.com/FuelLabs/fuelup) in order to install the `forc-index` binary that we'll use to develop our index.

### 1.1 Install `fuelup`

To Install fuelup with the default features/options, use the following command, which downloads the fuelup installation script and runs it interactively.

```bash
curl \
  --proto '=https' \
  --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

> If you require a non-default `fuelup` installation, please [read the `fuelup` installation docs.](https://github.com/FuelLabs/fuelup)

## 2. Using the `forc-index` plugin

The primary means of interfacing with the Fuel indexer **for index development** is the [`forc-index` CLI tool](https://crates.io/crates/forc-index). `forc-index` is a [`forc`](https://github.com/FuelLabs/sway/tree/master/forc) plugin specifically created to interface with the Fuel indexer service. Since we already installed `fuelup` in a previous step <sup>[1.1](#11-install-fuelup)</sup>, we should be able to check that our `forc-index` binary was successfully installed and added to our `PATH`.

```bash
which forc-index
```

```text
/Users/me/.fuelup/bin/forc-index
```

> IMPORTANT: `fuelup` will install several binaries from the Fuel ecosystem and add them into your `PATH`, including the `fuel-indexer` binary. The `fuel-indexer` binary is the primary binary that users can use to spin up a Fuel indexer service.

```bash
which fuel-indexer
```

```text
/Users/me/.fuelup/bin/fuel-indexer
```

### 2.1 Check for components

Once the `forc-index` plugin is installed, let's go ahead and see what indexer components we have installed.

> Many of these components are required for development work (e.g., `fuel-core`, `psql`) but some are even required for non-development usage as well (e.g., `wasm-snip`, `fuelup`).

```bash
forc index check
```

```text
+--------+------------------------+---------------------------------------------------------+
| Status |       Component        |                         Details                         |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | fuel-indexer binary    |  /Users/me/.fuelup/bin/fuel-indexer                     |
+--------+------------------------+---------------------------------------------------------+
|   ⛔️   | fuel-indexer service   |  Failed to detect service at Port(29987).               |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | psql                   |  /usr/local/bin/psql                                    |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | fuel-core              |  /Users/me/.fuelup/bin/fuel-core                        |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | docker                 |  /usr/local/bin/docker                                  |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | fuelup                 |  /Users/me/.fuelup/bin/fuelup                           |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | wasm-snip              |  /Users/me/.cargo/bin/wasm-snip                         |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | forc-postgres          |  /Users/me/.fuelup/bin/fuelup                           |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | rustc                  |  /Users/me/.cargo/bin/rustc                             |
+--------+------------------------+---------------------------------------------------------+
```

### 2.2 Setup a Database and Start the Indexer Service

To quickly setup and bootstrap the PostgreSQL database that we'll need, we'll use `forc-index` and its `forc-postgres` sub-plugin, which was also installed wtih `fuelup`.

We can quickly create a bootstrapped database and start the Fuel indexer service by running the following command:

> IMPORTANT: Ensure that any local PostgreSQL instance that is running on port `5432` is stopped.

```bash
forc index start \
    --auto-database-setup                       # Setup and start a default database.
    --fuel-node-host node-beta-2.fuel.network \ # Connect to a Fuel node at this host
    --fuel-node-port 80                         # and port, and monitor the network.
```

You should see output indicating the successful creation of a database and start of the indexer service; there may be much more content in your session, but it should generally contain output similar to the following lines:

```text
📦 Downloading, unpacking, and bootstrapping database...

▹▹▸▹▹ ⏱  Setting up database...

💡 Creating database at 'postgres://postgres:postgres@localhost:5432/postgres'

✅ Successfully created database at 'postgres://postgres:postgres@localhost:5432/postgres'.

✅ Successfully started database at 'postgres://postgres:postgres@localhost:5432/postgres'.

✅ Successfully started the indexer service.
```

> You can `Ctrl+C` to exit the `forc index start` process, and your indexer service and database should still be running in the background.

### 2.3 Creating a new index

Now that we have our development environment set up, the next step is to create an index.

```bash
forc index new hello-index --namespace my_project && cd hello-index
```

> The `namespace` of your project is a required option. You can think of a `namespace` as your organization name or company name. Your index project might contain one or many indices all under the same `namespace`.

```text
forc index new hello-index --namespace my_project

███████╗██╗   ██╗███████╗██╗         ██╗███╗   ██╗██████╗ ███████╗██╗  ██╗███████╗██████╗
██╔════╝██║   ██║██╔════╝██║         ██║████╗  ██║██╔══██╗██╔════╝╚██╗██╔╝██╔════╝██╔══██╗
█████╗  ██║   ██║█████╗  ██║         ██║██╔██╗ ██║██║  ██║█████╗   ╚███╔╝ █████╗  ██████╔╝
██╔══╝  ██║   ██║██╔══╝  ██║         ██║██║╚██╗██║██║  ██║██╔══╝   ██╔██╗ ██╔══╝  ██╔══██╗
██║     ╚██████╔╝███████╗███████╗    ██║██║ ╚████║██████╔╝███████╗██╔╝ ██╗███████╗██║  ██║
╚═╝      ╚═════╝ ╚══════╝╚══════╝    ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝

An easy-to-use, flexible indexing service built to go fast. 🚗💨

----

Read the Docs:
- Fuel Indexer: https://github.com/FuelLabs/fuel-indexer
- Fuel Indexer Book: https://fuellabs.github.io/fuel-indexer/latest
- Sway Book: https://fuellabs.github.io/sway/latest
- Rust SDK Book: https://fuellabs.github.io/fuels-rs/latest

Join the Community:
- Follow us @SwayLang: https://twitter.com/fuellabs_
- Ask questions in dev-chat on Discord: https://discord.com/invite/xfpK4Pe

Report Bugs:
- Fuel Indexer Issues: https://github.com/FuelLabs/fuel-indexer/issues/new

Take a quick tour.
`forc index check`
    List indexer components.
`forc index new`
    Create a new index.
`forc index init`
    Create a new index in an existing directory.
`forc index start`
    Start a local indexer service.
`forc index build`
    Build your index.
`forc index deploy`
    Deploy your index.
`forc index remove`
    Stop a running index.
```

> IMPORTANT: If you want more details on how this index works, check out our [block explorer index example](https://fuellabs.github.io/fuel-indexer/master/examples/block-explorer.html).

### 2.4 Deploying our index

At this point, we have a brand new index that will index some blocks and transactions. And with our database and Fuel indexer service up and running, all that's left is to build and deploy the index in order to see it in action. but now we need to build and deploy it in order to see it in action.

```bash
forc index deploy --manifest hello_index.manifest.yaml
```

If all goes well, you should see the following:

```text
▹▹▸▹▹ ⏰ Building...                                                                                         Finished dev [unoptimized + debuginfo] target(s) in 0.87s
▪▪▪▪▪ ✅ Build succeeded.

Deploying index at hello_index.manifest.yaml to http://127.0.0.1:29987/api/index/my_project/hello_index
▹▸▹▹▹ 🚀 Deploying...
{
  "assets": [
    {
      "digest": "79e74d6a7b68a35aeb9aa2dd7f6083dae5fdba5b6a2f199529b6c49624d1e27b",
      "id": 1,
      "index_id": 1,
      "version": 1
    },
    {
      "digest": "4415628d9ea79b3c3f1e6f02b1af3416c4d0b261b75abe3cc81b77b7902549c5",
      "id": 1,
      "index_id": 1,
      "version": 1
    },
    {
      "digest": "e901eba95ce8b4d1c159c5d66f24276dc911e87dbff55fb2c10d8b371528eacc",
      "id": 1,
      "index_id": 1,
      "version": 1
    }
  ],
  "success": "true"
}
▪▪▪▪▪ ✅ Successfully deployed index.
```

## 3. Querying for data

With our index deployed, we should be able to query for newly indexed data after a few seconds.

Below, we write a simple GraphQL query that simply returns a few fields from all transactions that we've indexed.

```bash
curl -X POST http://127.0.0.1:29987/api/graph/my_project/hello_index \
   -H 'content-type: application/json' \
   -d '{"query": "query { tx { id hash block }}", "params": "b"}' \
| json_pp
```

```text
[
   {
      "block" : 7017844286925529648,
      "hash" : "fb93ce9519866676813584eca79afe2d98466b3e2c8b787503b76b0b4718a565",
      "id" : 7292230935510476086,
   },
   {
      "block" : 3473793069188998756,
      "hash" : "5ea2577727aaadc331d5ae1ffcbc11ec4c2ba503410f8edfb22fc0a72a1d01eb",
      "id" : 4136050720295695667,
   },
   {
      "block" : 7221293542007912803,
      "hash" : "d2f638c26a313c681d75db2edfbc8081dbf5ecced87a41ec4199d221251b0578",
      "id" : 4049687577184449589,
   },
]
```

### Finished! 🥳

Congrats, you just created, built, and deployed your first index on the world's fastest execution layer. For more detailed info on how the Fuel indexer service works, make sure you [**read the book**](https://fuellabs.github.io/fuel-indexer/master/).

## `forc-index` Plugin

The primary way of developing Fuel indexers for end users is via the `forc-index` plugin. The `forc-index` plugin, is a CLI tool that is bundled with Fuel's primary CLI tooling interface, [`forc`](https://github.com/FuelLabs/sway/tree/master/forc) ("Fuel Orchestrator").

As mentioned in the [dependencies](#dependencies) section, the `forc-index` plugin is made available once you download [`fuelup`](#fuelup).

If you've successfully gone through the [Quickstart](#quickstart), you should already have `forc-index` installed and available in your `PATH`.

### `forc index check`

Check to see which indexer components you have installed.

```bash
forc index check
```

### `forc index new`

Create new index project at the provided path.

```bash
forc index new --namespace my_org_name
```

### `forc index init`

Create a new index project at the provided path. If no path is provided the current working directory will be used.

```bash
forc index init --namespace my_org_name
```

### `forc index build`

Build the index in the current directory.

```bash
forc index build --manifest my_index.manifest.yaml
```

### `forc index deploy`

Deploy a given index project to a particular endpoint

```bash
forc index deploy --url https://index.swaysway.io --manifest my_index.manifest.yaml
```

### `forc index remove`

Kill a running indexer.

```bash
forc index remove --url https://index.swayswap.io --manifest my_index.manifest.yaml
```

### `forc index start`

Start the indexer service.

```bash
forc index start
```

## Schema

The Fuel indexer uses data models derived from GraphQL schema types in order to persist data to a database backend.

```graphql
schema {
    query: QueryRoot
}

type QueryRoot {
    account: Account
}

type Account {
    id: ID!
    address: Address! @unique
}
```

## Modules

Within the context of the Fuel indexer, WebAssembly (WASM) modules are binaries that are compiled to a `wasm32-unknown-unknown` target, which can then be deployed to a running indexer service.

### Notes on WASM modules

There are a few points that Fuel indexer users should know when using WASM:

1. WASM modules are only used if the execution mode specified in your manifest file is `wasm`.

2. Developers should be aware of what things may not work off-the-shelf in a module: file I/O, thread spawning, and anything that depends on system libraries. This is due to the technological limitations of WASM as a whole; more information can be found [here](https://rustwasm.github.io/docs/book/reference/which-crates-work-with-wasm.html).

3. As of this writing, there is a small bug in newly built Fuel indexer WASM modules that produces a WASM runtime error due to an errant upstream dependency. For now, a quick workaround requires the use of `wasm-snip` to remove the errant symbols from the WASM module. More info can be found in the related script [here](https://github.com/FuelLabs/fuel-indexer/blob/master/scripts/stripper.bash).

> IMPORTANT: Users on Apple Silicon macOS systems may experience trouble when trying to build WASM modules due to its `clang` binary not supporting WASM targets. If encountered, you can install a binary with better support from Homebrew (`brew install llvm`) and instruct `rustc` to leverage it by setting the following environment variables:
>
> - `AR=/opt/homebrew/opt/llvm/bin/llvm-ar`
> - `CC=/opt/homebrew/opt/llvm/bin/clang`

# For Contributors

Contributors of the Fuel indexer project are devs looking to help  backends for their dApps.

## Dev Dependencies

### `docker`

> IMPORTANT: Docker is not required to run the Fuel indexer.

- We use Docker to produce reproducible environments for users that may be concerned with installing components with large sets of dependencies (e.g. PostgreSQL).
- Docker can be downloaded [here](https://docs.docker.com/engine/install/).

### Database

At this time, the Fuel indexer requires the use of a database. We currently support a single database option: PostgreSQL. PostgreSQL is a database solution with a complex feature set and requires a database server.

#### PostgreSQL

> Note: The following explanation is for demonstration purposes only. A production setup should use secure users, permissions, and passwords.

On macOS systems, you can install PostgreSQL through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). Once installed, you can add PostgreSQL to your system by running `brew install postgresql`. You can then start the service through `brew services start postgresql`. You'll need to create a database for your index data, which you can do by running `createdb [DATABASE_NAME]`. You may also need to create the `postgres` role; you can do so by running `createuser -s postgres`.

For Linux-based systems, the installation process is similar. First, you should install PostgreSQL according to your distribution's instructions. Once installed, there should be a new `postgres` user account; you can switch to that account by running `sudo -i -u postgres`. After you have switched accounts, you may need to create a `postgres` database role by running `createuser --interactive`. You will be asked a few questions; the name of the role should be `postgres` and you should elect for the new role to be a superuser. Finally, you can create a database by running `createdb [DATABASE_NAME]`.

In either case, your PostgreSQL database should now be accessible at `postgres://postgres@127.0.0.1:5432/[DATABASE_NAME]`.

### SQLx

- After setting up your database, you should install `sqlx-cli` in order to run migrations for your indexer service.
- You can do so by running `cargo install sqlx-cli --features postgres`.
- Once installed, you can run the migrations by running the following command after changing `DATABASE_URL` to match your setup.

## Building from Source

### Clone repository

```bash
git clone git@github.com:FuelLabs/fuel-indexer.git && cd fuel-indexer/
```

### Run migrations

#### PostgreSQL migrations

```sh
cd packages/fuel-indexer-database/postgres
DATABASE_URL=postgres://postgres@localhost sqlx migrate run
```

### Start the service

```bash
cargo run --bin fuel-indexer
```

> If no configuration file or other options are passed, the service will default to a `postgres://postgres@localhost` database connection.

## Testing

Fuel indexer tests are currently broken out by a database feature flag. In order to run tests with a PostgreSQL backend, use `--features postgres`.

Further, the indexer uses end-to-end (E2E) tests. In order to trigger these end-to-end tests, you'll want to use the `e2e` features flag: `--features e2e`.

> All end-to-end tests also require the use of a database feature. For example, to run the end-to-end tests with a Posgres backend, use `--features e2e,postgres`.

### Default tests

```bash
cargo test --locked --workspace --all-targets
```

### End-to-end tests

```bash
cargo test --locked --workspace --all-targets --features e2e,postgres
```

### `trybuild` tests

For tests related to the meta-programming used in the Fuel indexer, we use `trybuild`.

```bash
RUSTFLAGS='-D warnings' cargo test -p fuel-indexer-macros --locked
```

## Contributing

If you're interested in contributing PRs to make the Fuel indexer a better project, feel free to read [our contributors document](./CONTRIBUTING.md).

# Read the book

Whether you're a user or a contributor, for more detailed info on how the Fuel indexer service works, make sure you [**read the book**](https://fuellabs.github.io/fuel-indexer/master/).
