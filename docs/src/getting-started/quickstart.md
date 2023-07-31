# Quickstart

In this tutorial you will:

1. Bootstrap your development environment.
2. Create, build, and deploy an indexer to an indexer service hooked up to Fuel's `beta-3` testnet.
3. Query your newly created index for data using GraphQL.

## 1. Setting up your environment

In this Quickstart, we'll use Fuel's toolchain manager [`fuelup`](https://github.com/FuelLabs/fuelup) in order to install the `forc-index` component that we'll use to develop our indexer.

### 1.1 Install `fuelup`

To install fuelup with the default features/options, use the following command to download the fuelup installation script and run it interactively.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://install.fuel.network/fuelup-init.sh | sh
```

> If you require a non-default `fuelup` installation, please [read the `fuelup` installation docs.](https://github.com/FuelLabs/fuelup)

### 1.2 WebAssembly (WASM) Setup

Indexers are typically compiled to WASM so you'll need to have the proper WASM compilation target available on your system. You can install this target using `rustup`:

```
rustup target add wasm32-unknown-unknown
```

Additionally, you'll need the `wasm-snip` utility in order to remove errant symbols from your compiled WASM binary. You can install this tool using `cargo`:

```
cargo install wasm-snip
```

## 2. Using the `forc-index` plugin

The primary means of interfacing with the Fuel indexer for indexer development is the [`forc-index` CLI tool](https://crates.io/crates/forc-index). `forc-index` is a [`forc`](https://github.com/FuelLabs/sway/tree/master/forc) plugin specifically created to interface with the Fuel indexer service. Since we already installed `fuelup` in a previous step <sup>[1.1](#11-install-fuelup)</sup>, we should be able to check that our `forc-index` binary was successfully installed and added to our `PATH`.

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

To quickly setup and bootstrap the PostgreSQL database that we'll need, we'll use `forc index`.

We can quickly create a bootstrapped database and start the Fuel indexer service by running the following command:

> IMPORTANT: Below we're specifying our Postgres hostname as `--postgres-host postgresql`, but you will need to be specific to your own Postgres instance details (see `forc index start --help` for more details). You can try using the `--embedded-database` flag in order to quickly use an embedded instance of Postgres, but this is flaky and often depends on what platform you're using.

```bash
forc index start --fuel-node-host beta-3.fuel.network --fuel-node-port 80 --run-migrations --postgres-host postgresql
```

You should see output indicating the successful creation of a database and start of the indexer service; there may be much more content in your session, but it should generally contain output similar to the following lines:

```text
✅ Successfully started the indexer service at PID 39407

2023-07-31T15:57:28.942954Z  INFO fuel_indexer::commands::run: 109: Configuration: IndexerConfig { metering_points: Some(30000000000), log_level: "info", verbose: false, local_fuel_node: false, indexer_net_config: false, fuel_node: FuelClientConfig { host: "beta-3.fuel.network", port: "80" }, web_api: WebApiConfig { host: "localhost", port: "29987", max_body_size: 5242880 }, database: PostgresConfig { user: "postgres", password: "XXXX", host: "localhost", port: "5432", database: "postgres", verbose: "false" }, metrics: false, stop_idle_indexers: false, run_migrations: true, authentication: AuthenticationConfig { enabled: false, strategy: None, jwt_secret: "XXXX", jwt_issuer: None, jwt_expiry: None }, rate_limit: RateLimitConfig { enabled: false, request_count: None, window_size: None }, replace_indexer: false, accept_sql_queries: false }
2023-07-31T15:57:28.948657Z  INFO sqlx::postgres::notice: 157: relation "_sqlx_migrations" already exists, skipping
2023-07-31T15:57:28.976258Z  INFO fuel_indexer::service: 378: Resuming Indexer(fuel.indexer_test) from block 81188
2023-07-31T15:57:29.077928Z  INFO fuel_indexer::database: 187: Loading schema for Indexer(fuel.indexer_test) with Version(2738d221cf1e926d28e62bc93604a96ec6f7c5093e766f45a4555ed06e437b7f).
2023-07-31T15:57:29.081302Z  WARN fuel_indexer::executor: 87: No end_block specified in manifest. Indexer will run forever.
2023-07-31T15:57:29.081311Z  INFO fuel_indexer::executor: 109: Indexer(fuel.indexer_test) subscribing to Fuel node at beta-3.fuel.network:80
2023-07-31T15:57:29.081424Z  INFO fuel_indexer::service: 194: Registered Indexer(fuel.indexer_test)
2023-07-31T15:57:29.082150Z  INFO fuel_indexer_lib::utils: 132: Parsed SocketAddr '127.0.0.1:29987' from 'localhost:29987
```

### 2.3 Creating a new indexer

Now that we have our development environment set up, the next step is to create an indexer.

```bash
forc index new hello-indexer --namespace FuelLabs && cd hello-indexer
```

> The `namespace` of your project is a required option. You can think of a `namespace` as your organization name or company name. Your project might contain one or many indexers all under the same `namespace`. For a complete list of options passed to `forc index new`, see [here](../forc-index/new.md)

```text
forc index new hello-indexer --namespace FuelLabs

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
`forc index auth`
    Authenticate against an indexer service.
`forc index status`
    Check the status of an indexer.
```

### 2.4 Deploying our indexer

At this point, we have a brand new indexer that will index some blocks and transactions. And with both our database and Fuel indexer services up and running, all that's left is to build and deploy the indexer in order to see it in action. Let's build and deploy our indexer:

```bash
forc index deploy
```

> IMPORTANT: `forc index deploy` by defaults runs `forc index build` prior to deploying the indexer. The same result can be produced by running `forc index build` then subsequently running `forc index deploy`.

If all goes well, you should see the following:

```text
▹▹▹▹▹ ⏰ Building...                         Finished dev [unoptimized + debuginfo] target(s) in 0.96s
▪▪▪▪▪ ✅ Build succeeded.                    Deploying indexer
▪▪▪▪▪ ✅ Successfully deployed indexer.
```

## 3. Querying for data

With our indexer deployed, we should be able to query for newly indexed data after a few seconds.

Below, we write a simple GraphQL query that simply returns a few fields from all transactions that we've indexed.

You can open your GraphQL query playground at http://127.0.0.1:29987/api/playground/fuellabs/hello_indexer and submit the following GraphQL query.

```graphql
query {
  tx {
    id
    hash
    block
  }
}
```

The response you get should resemble:

```json
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

Congrats, you just created, built, and deployed your first indexer on the world's fastest execution layer.
