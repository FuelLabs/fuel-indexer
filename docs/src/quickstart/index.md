# Quickstart

In this Quickstart tutorial we'll go over:

1. How to bootstrap an environment for use with a Fuel indexer.
2. How to use the `forc index` plugin to create, build, and deploy indicies.
3. How to query our newly created indices for data using GraphQL.

> IMPORTANT: Docker is a prerequisite for using this Quickstart. If Docker is not installed on your machine, please review the Docker installation instructions [here](https://docs.docker.com/engine/install/).

## 1. Setting up your environment

As a user of the Fuel indexer, you shouldn't require _too many_ dependencies. The indexer binaries available via `fuelup` are precompiled with all required dependencies, and optimized for size.

In this Quickstart, we'll only be using Postgres. Further, instead of going through the
drudgery of installing Postgres on your local machine, we'll just use [the `latest`
Postgres docker image](https://hub.docker.com/_/postgres) via the following command: `docker pull postgres`.

## 2. Using the `forc-index` plugin

- The primary means of interfacing with the Fuel indexer **for development** is the [`forc-index` CLI tool](https://crates.io/crates/forc-index).
- `forc-index` is a [`forc`](https://github.com/FuelLabs/sway/tree/master/forc) plugin specifically for the Fuel indexer service.
- For convenience, the `forc-index` binary is already included with `fuelup`.
  - [`fuelup` setup instructions](https://github.com/FuelLabs/fuelup)

> IMPORTANT: `fuelup` also contains the `fuel-indexer` binary. This is the binary that runs the Fuel indexer service.

If your `forc-index` and `fuel-indexer` binaries were successfully installed via `fuelup`, your `PATH` should contian:

```text
➜  which forc-index

/Users/me/.fuelup/bin/forc-index

➜  which fuel-indexer

/Users/me/.fuelup/bin/fuel-indexer
```

### 2.1 Components check

Once the `forc-index` plugin is installed, let's go ahead and see what indexer components we have installed.

> IMPORTANT: Many of these components are required for development work (e.g., `fuel-core`, `psql`) but some are even
> required for non-development usage as well (e.g., `wasm-snip`, `fuelup`)

```text
➜  forc index check

❌ Could not connect to indexers service: error sending request for url (http://127.0.0.1:29987/api/health): error trying to connect: tcp connect error: Connection refused (os error 61)

+--------+------------------------+----------------------------------------------------------------------------+
| Status |       Component        |                                  Details                                   |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuel-indexer binary    |  Found 'fuel-indexer' at '/Users/rashad/.fuelup/bin/fuel-indexer'          |
+--------+------------------------+----------------------------------------------------------------------------+
|   ⛔️   | fuel-indexer service   |  Failed to detect a locally running fuel-indexer service at Port(29987).   |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | psql                   |  Found 'psql' at '/usr/local/bin/psql'                                     |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | sqlite                 |  Found 'sqlite' at '/usr/bin/sqlite3'                                      |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuel-core              |  Found 'fuel-core' at '/Users/rashad/.fuelup/bin/fuel-core'                |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | docker                 |  Found 'docker' at '/usr/local/bin/docker'                                 |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuelup                 |  Found 'fuelup' at '/Users/rashad/.fuelup/bin/fuelup'                      |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | wasm-snip              |  Found 'wasm-snip' at '/Users/rashad/.cargo/bin/wasm-snip'                 |
+--------+------------------------+----------------------------------------------------------------------------+
```

### 2.2 Creating a new index

Once we're sure we have the right components installed (shown above 👆🏽) we'll create a new index.

```bash
➜  forc index new hello-index --namespace my_project

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
`forc index build
    Build your index.
`forc index deploy`
    Deploy your index.
`forc index stop`
    Stop a running index.
```

> IMPORTANT: If you want more details on how this index works, checkout or [Block Explorer index example](https://fuellabs.github.io/fuel-indexer/master/examples/block-explorer.html).

### 2.3 Deploying our index

By now we have a brand new index that will index some blocks and transactions, but now we need to build and deploy it in order to see it in action.

#### 2.3.1 Starting an indexer service

- To start an indexer service, we'll be spinning up Postgres and Fuel indexer containers via docker compose. Our indexer service will connect to Fuel's `beta-2` network so that we can index blocks and transactions from an _actual_ Fuel node. We'll use the docker compose file below, and spinning everything up with `docker compose up`.

> IMPORTANT: Ensure that any local Postgres instance that is running on port `5432` is stopped.
>
> You can open up a `docker-compose.yaml` file in the same directory as your index project, and paste the YAML content below to this file.

```text
version: "3.9"
services:
  postgres:
    image: postgres:latest
    ports:
      - "5432:5432"
    volumes:
      - .:/usr/local/postgres
    environment:
      - POSTGRES_PASSWORD=postgres
      - PGUSER=postgres
    healthcheck:
      test: ["CMD-SHELL", "pg_isready", "-U", "postgres", "-d", "postgres"]
      interval: 30s
      timeout: 60s
      retries: 5
      start_period: 80s
  fuel-indexer:
    image: ghcr.io/fuellabs/fuel-indexer:latest
    command: bash -c "sleep 2 && ./fuel-indexer --fuel-node-host node-beta-2.fuel.network --fuel-node-port 80 --postgres-host postgres --postgres-password postgres --graphql-api-host 0.0.0.0"
    ports:
      - "29987:29987"
    volumes:
      - .:/usr/local/fuel-indexer
    depends_on:
      - postgres
```

#### 2.3.2 Deploying your index to your service

With our database and indexer service containers up and running, we'll deploy the index that we previously created. If all goes well, you should see the following:

```bash
➜  cd hello-index

➜  forc-index deploy --manifest hello_index.manifest.yaml --url http://0.0.0.0:29987
▹▹▸▹▹ ⏰ Building...                                                                                         Finished dev [unoptimized + debuginfo] target(s) in 0.87s
▪▪▪▪▪ ✅ Build succeeded.

Deploying index at tina.manifest.yaml to http://127.0.0.1:29987/api/index/my_project/hello_index
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

With our index deployed, after a few seconds, we should be able to query for newly indexed data.

```bash
➜ curl -X POST http://0.0.0.0:29987/api/graph/my_project \
   -H 'content-type: application/json' \
   -d '{"query": "query { tx { id hash status block }}", "params": "b"}' \
| json_pp
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   364  100   287  100    77   6153   1650 --:--:-- --:--:-- --:--:--  9100
[
   {
      "block" : 7017844286925529648,
      "hash" : "fb93ce9519866676813584eca79afe2d98466b3e2c8b787503b76b0b4718a565",
      "id" : 7292230935510476086,
      "status" : {
         "block" : "0x8c34daaa2c58629cb98fa66d4f5ce0c0850d24e655ed6006b22204dac42fd918",
         "status" : "success",
         "time" : "2022-11-10 14:35:58 UTC"
      }
   },
   {
      "block" : 3473793069188998756,
      "hash" : "5ea2577727aaadc331d5ae1ffcbc11ec4c2ba503410f8edfb22fc0a72a1d01eb",
      "id" : 4136050720295695667,
      "status" : {
         "block" : "0x2b892dd6574e4a803f90c85754d6c8e154ec5f7dd91a25ce962820dce12f15e5",
         "status" : "success",
         "time" : "2022-11-10 13:35:58 UTC"
      }
   },
   {
      "block" : 7221293542007912803,
      "hash" : "d2f638c26a313c681d75db2edfbc8081dbf5ecced87a41ec4199d221251b0578",
      "id" : 4049687577184449589,
      "status" : {
         "block" : "0xa0812af8738da14f7db2f00a53341492aa339f8d88e118820e78a500b11e3560",
         "status" : "success",
         "time" : "2022-11-10 12:35:58 UTC"
      }
   },
]
```

### Finsihed 🥳

Congrats, you just created, built, and deployed your first index on the world's fastest execution layer. For more detailed info on how the Fuel indexer service works, make sure you [**read the book**](https://fuellabs.github.io/fuel-indexer/master/).
