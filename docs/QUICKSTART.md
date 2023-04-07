# Quickstart

In this tutorial you will:

1. Bootstrap your development environment.
2. Create, build, and deploy an index to an indexer service hooked up to Fuel's `beta-3` testnet.
3. Query the indexer service for indexed data using GraphQL.

## 1. Setting up your environment

In this Quickstart, we'll use Docker's Compose to spin up a Fuel indexer service with a PostgreSQL database backend. We will also use Fuel's toolchain manager [`fuelup`](https://github.com/FuelLabs/fuelup) in order to install the `forc-index` binary that we'll use to develop our index.

### 1.1 Install `fuelup`

To install fuelup with the default features/options, use the following command, which downloads the fuelup installation script and runs it interactively.

```bash
curl \
  --proto '=https' \
  --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

> If you require a non-default `fuelup` installation, please [read the `fuelup` installation docs.](https://github.com/FuelLabs/fuelup)

### 1.2 WebAssembly (WASM) Setup

Indexers are typically compiled to WASM and thus you'll need to have the proper WASM compilation target available on your system. You can install it through `rustup`:

```
rustup target add wasm32-unknown-unknown
```

Additionally, you'll need the `wasm-snip` utility in order to shrink the WASM binary size and cut out errant symbols. You can install it through `cargo`:

```
cargo install wasm-snip
```

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
|   ⛔️   | fuel-indexer binary    |  Can't locate fuel-indexer.                             |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | fuel-indexer service   |  Local service found: PID(63967) | Port(29987).         |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | psql                   |  /usr/local/bin/psql                                    |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | fuel-core              |  /Users/me/.cargo/bin/fuel-core                         |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | docker                 |  /usr/local/bin/docker                                  |
+--------+------------------------+---------------------------------------------------------+
|   ⛔️   | fuelup                 |  Can't locate fuelup.                                   |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | wasm-snip              |  /Users/me/.cargo/bin/wasm-snip                         |
+--------+------------------------+---------------------------------------------------------+
|   ⛔️   | forc-postgres          |  Can't locate fuelup.                                   |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | rustc                  |  /Users/me/.cargo/bin/rustc                             |
+--------+------------------------+---------------------------------------------------------+
|   ✅   | forc-wallet            |  /Users/me/.cargo/bin/forc-wallet                       |
+--------+------------------------+---------------------------------------------------------+
```

### 2.2 Setup a Database and Start the Indexer Service

To quickly setup and bootstrap the PostgreSQL database that we'll need, we'll use `forc index` and its `forc index postgres` subcommand.

We can quickly create a bootstrapped database and start the Fuel indexer service by running the following command:

> IMPORTANT: Ensure that any local PostgreSQL instance that is running on port `5432` is stopped.

```bash
forc index start \
    --embedded-database
    --fuel-node-host node-beta-2.fuel.network \
    --fuel-node-port 80
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

### 2.3 Creating a new indexer

Now that we have our development environment set up, the next step is to create an indexer.

```bash
forc index new hello-indexer --namespace my_project && cd hello-indexer
```

> The `namespace` of your project is a required option. You can think of a `namespace` as your organization name or company name. Your project might contain one or many indexers all under the same `namespace`.

```text
forc index new hello-indexer --namespace my_project

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
    Create a new indexer.
`forc index init`
    Create a new indexer in an existing directory.
`forc index start`
    Start a local indexer service.
`forc index build`
    Build your indexer.
`forc index deploy`
    Deploy your indexer.
`forc index remove`
    Stop a running indexer.
`forc index revert`
    Revert a deployed indexer.
`forc index auth`
    Authenticate against an indexer service.
```

> IMPORTANT: If you want more details on how this indexer works, check out our [block explorer indexer example](https://fuellabs.github.io/fuel-indexer/master/examples/block-explorer.html).

### 2.4 Deploying our indexer

At this point, we have a brand new indexer that will index some blocks and transactions. And with our database and Fuel indexer service up and running, all that's left is to build and deploy the indexer in order to see it in action. but now we need to build and deploy it in order to see it in action.

```bash
forc index deploy
```

If all goes well, you should see the following:

```text
▹▹▹▹▹ ⏰ Building...                         Finished dev [unoptimized + debuginfo] target(s) in 0.96s
▪▪▪▪▪ ✅ Build succeeded.                    Deploying indexer
▪▪▪▪▪ ✅ Successfully deployed indexer.
```

## 3. Querying for data

With our indexer deployed, we should be able to query for newly indexed data after a few seconds.

Below, we write a simple GraphQL query that simply returns a few fields from all transactions that we've indexed.

```bash
curl -X POST -H "Content-Type: application/graphql" 
--data '{ "query": "query { tx { id, hash, block } }" }' 
http://127.0.0.1:29987/api/graph/my_project/hello_indexer
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

Congrats, you just created, built, and deployed your first indexer on the world's fastest execution layer. For more detailed info on how the Fuel indexer service works, make sure you [**read the book**](https://fuellabs.github.io/fuel-indexer/master/).
