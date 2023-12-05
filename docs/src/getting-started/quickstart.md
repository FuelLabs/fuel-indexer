# Quickstart

In this tutorial you will:

1. Bootstrap your development environment.
2. Create, build, and deploy an indexer to an indexer service hooked up to Fuel's `beta-4` testnet.
3. Query your indexer's newly created index for data using GraphQL.

---

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

```bash
rustup target add wasm32-unknown-unknown
```

Additionally, you'll need the `wasm-snip` utility in order to remove errant symbols from your compiled WASM binary. You can install this tool using `cargo`:

```bash
cargo install wasm-snip
```

---

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

> IMPORTANT: Below we're specifying our Postgres `hostname` as `--postgres-host postgresql`, but you might need to change this based on your own Postgres instance details (see `forc index start --help` for more details).
>
> Additionally, you can try using the `--embedded-database` flag in order to quickly use an embedded instance of Postgres, but this flag can be flaky, and its ease of use often depends on what platform you're using.
>
> If you find that `--embedded-database` isn't working on your machine (for whatever reason), we strongly recommend that you simply used the Dockerized Fuel indexer components included in the project by using the `docker compose` script [included in the project](https://github.com/FuelLabs/fuel-indexer/blob/develop/scripts/docker-compose.yaml).

```bash
forc index start --network beta-4 --run-migrations --postgres-host postgresql
```

You should see output indicating the successful creation of a database and start of the indexer service; there may be much more content in your session, but it should generally contain output similar to the following lines:

```text
✅ Successfully started the indexer service at PID 39407

2023-07-31T15:57:28.942954Z  INFO fuel_indexer::commands::run: 109: Configuration: IndexerConfig { metering_points: Some(30000000000), log_level: "info", verbose: false, local_fuel_node: false, indexer_net_config: false, fuel_node: FuelClientConfig { host: "beta-4.fuel.network", port: "80" }, web_api: WebApiConfig { host: "localhost", port: "29987", max_body_size: 5242880 }, database: PostgresConfig { user: "postgres", password: "XXXX", host: "localhost", port: "5432", database: "postgres", verbose: "false" }, metrics: false, stop_idle_indexers: false, run_migrations: true, authentication: AuthenticationConfig { enabled: false, strategy: None, jwt_secret: "XXXX", jwt_issuer: None, jwt_expiry: None }, rate_limit: RateLimitConfig { enabled: false, request_count: None, window_size: None }, replace_indexer: false, accept_sql_queries: false }
2023-07-31T15:57:28.948657Z  INFO sqlx::postgres::notice: 157: relation "_sqlx_migrations" already exists, skipping
2023-07-31T15:57:28.976258Z  INFO fuel_indexer::service: 378: Resuming Indexer(fuel.indexer_test) from block 81188
2023-07-31T15:57:29.077928Z  INFO fuel_indexer::database: 187: Loading schema for Indexer(fuel.indexer_test) with Version(2738d221cf1e926d28e62bc93604a96ec6f7c5093e766f45a4555ed06e437b7f).
2023-07-31T15:57:29.081302Z  WARN fuel_indexer::executor: 87: No end_block specified in manifest. Indexer will run forever.
2023-07-31T15:57:29.081311Z  INFO fuel_indexer::executor: 109: Indexer(fuel.indexer_test) subscribing to Fuel node at beta-4.fuel.network:80
2023-07-31T15:57:29.081424Z  INFO fuel_indexer::service: 194: Registered Indexer(fuel.indexer_test)
2023-07-31T15:57:29.082150Z  INFO fuel_indexer_lib::utils: 132: Parsed SocketAddr '127.0.0.1:29987' from 'localhost:29987
```

### 2.3 Creating a new indexer

Now that we have our development environment set up, the next step is to create an indexer.

```bash
forc index new hello-indexer --namespace fuellabs && cd hello-indexer
```

> The `namespace` of your project is a required option. You can think of a `namespace` as your organization name or company name. Your project might contain one or many indexers all under the same `namespace`. For a complete list of options passed to `forc index new`, see [here](../forc-index/new.md).

```text
forc index new hello-indexer --namespace FuelLabs

✅ Successfully created indexer


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
- Fuel Indexer Book: https://docs.fuel.network/docs/indexer/
- Sway Book: https://docs.fuel.network/docs/sway/
- Rust SDK Book: https://rust.fuel.network


Join the Community:
- Follow us @Fuel: https://twitter.com/fuel_network
- Ask questions in dev-chat on Discord: https://discord.com/invite/xfpK4Pe

Report Bugs:
- Fuel Indexer Issues: https://github.com/FuelLabs/fuel-indexer/issues/new

Take a quick tour.

`forc index auth`
    Authenticate against an indexer service.
`forc index build`
    Build an indexer.
`forc index check`
    List indexer components.
`forc index deploy`
    Deploy an indexer.
`forc index kill`
    Kill a running Fuel indexer process on a given port.
`forc index new`
    Create a new indexer.
`forc index remove`
    Stop a running indexer.
`forc index start`
    Start a local indexer service.
`forc index status`
    Check the status of an indexer.
```

### 2.4 Deploying our indexer

At this point, we have a brand new indexer that will index some blocks and transactions. And with both our database and Fuel indexer services up and running, all that's left to do is to build and deploy the indexer in order to see it in action. Let's build and deploy our indexer:

```bash
forc index deploy
```

> IMPORTANT: `forc index deploy` by defaults runs `forc index build` prior to deploying the indexer. The same result can be produced by running `forc index build` then subsequently running `forc index deploy`. For more info, checkout the [`forc index deploy`](./../forc-index/deploy.md) command.

If all goes well, you should see the following:

```text
▹▹▹▹▹ ⏰ Building...                         Finished dev [unoptimized + debuginfo] target(s) in 0.96s
▪▪▪▪▪ ✅ Build succeeded.                    Deploying indexer
▪▪▪▪▪ ✅ Successfully deployed indexer.
```

And we can check the status of our newly deployed indexer using:

```bash
forc index status
```

Which should show:

```text
✅ Successfully fetched service health:

client status: OK
database status: OK
uptime: 1m 30s

Indexers:

─ fuellabs
   └─ hello_world
      • id: 1
      • created at: 2023-11-08 15:09:49.205698 UTC (52s ago)
      • status: running
      • status message:
          Indexed 5440 blocks
```

> #### What is a "deployment" exactly?
>
> A _deployment_ within the context of Fuel's indexer is a series of steps taken to get your indexer project running in the wild.
>
> This series of steps involves compiling your indexer project to a `wasm32-unknown-unknown` target and uploading the indexer to a running Fuel indexer service. The service will then register an executor and build database tables for this indexer. Once this series of steps has completed, your indexer is considered to be ["deployed"](https://en.wikipedia.org/wiki/Software_deployment).
>
> Users will often find that they're simply deploying their indexers to a Fuel indexer service running on their local machine; this is just one valid use-case described in [our infrastructure docs](./indexer-service-infrastructure.md). Keep in mind that the intended use of a Fuel indexer service is as a standalone remote service that may run many different indexers at any given time.

## 3. Querying for data

With our indexer deployed, we should be able to query for newly indexed data after a few seconds.

Below, we write a simple GraphQL query that returns a few fields from all transactions that we've indexed.

You can open your GraphQL query playground at `http://127.0.0.1:29987/api/playground/fuellabs/hello_indexer` and submit the following GraphQL query.

```graphql
query {
  transaction {
    id,
    hash,
    block {
      id
    }
  }
}
```

The response you get should resemble:

```json
[
  {
    "block": {
      "id": "24002b29ef4331f5ee75a38bf6381f2c8e8d2d5b4d78470706dde7ab0b8d54c0"
    },
    "hash": "82b36dce26d926921b8e79597899d8712fdabf2553f28b45ef3851a968efb4b9",
    "id": "eb7e14822e18e71ba7c92c266b0976acda2344dfbef7a60099d400cc243394fb"
  },
  {
    "block": {
      "id": "1309ee2cb0846b1a7e45313e1c39b2a24ffd552a381f2f627225256f725a93e3"
    },
    "hash": "f0c7c778faa6eb2a8bf03c9c47bb3f836bd4fe37e69c18e30f853ff146522dcb",
    "id": "182b6343bbbca2fcecf97020ea3f3767b8f5c370a6b853d2add46853e542a113"
  },
  {
    "block": {
      "id": "95588e20296969a76576d519d301c6cabe1e009675e430da93e18ba2a0d38a49"
    },
    "hash": "e729045198ee10dcf49e431f50c2ffe8c37129cbe47e003a59aff81a88b03b50",
    "id": "6910ebc30a1037b83336c956c95f7fc470c4b76750a93f6a1f6d19a21d058b19"
  }
]
```

### Finished! 🥳

Congrats, you just created, built, and deployed your first indexer on the world's fastest execution layer.

For more info on how indexers work, please checkout the [reference guide](./../project-components/index.md).
