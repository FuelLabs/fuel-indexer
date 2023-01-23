# Quickstart

In this tutorial you will:

1. Bootstrap your development environment.
2. Create, build, and deploy an index to an indexer service hooked up to Fuel's `beta-2` testnet.
3. Query the indexer service for indexed data using GraphQL.

> IMPORTANT: Docker is a prerequisite for using this Quickstart. If Docker is not installed on your machine, please review the Docker installation instructions [here](https://docs.docker.com/engine/install/).

## 1. Setting up your environment

In this Quickstart, we'll use Docker's Compose to spin up a Fuel indexer service with a Postgres database backend. We will also use Fuel's toolchain manager [`fuelup`](https://github.com/FuelLabs/fuelup) in order to install the `forc-index` binary that we'll use to develop our index.

### 1.1 Install `fuelup`

To Install fuelup with the default features/options, use the following command, which downloads the fuelup installation script and runs it interactively.

```bash
curl \
  --proto '=https' \
  --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

> If you require a non-default `fuelup` installation, please [read the `fuelup` installation docs.](https://github.com/FuelLabs/fuelup)

### 1.2 Pull Docker images

We will use the `latest` Postgres and Fuel indexer images.

```bash
docker pull postgres:latest

docker pull ghcr.io/fuellabs/fuel-indexer:latest
```

## 2. Using the `forc-index` plugin

- The primary means of interfacing with the Fuel indexer **for index development** is the [`forc-index` CLI tool](https://crates.io/crates/forc-index).
- `forc-index` is a [`forc`](https://github.com/FuelLabs/sway/tree/master/forc) plugin specifically created to interface with the Fuel indexer service.
- Since we already installed `fuelup` in a previous step [1.1], we should be able to check that our `forc-index` binary was successfully installed and added to our `PATH`.

```bash
which forc index
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

> IMPORTANT: If you want more details on how this index works, checkout our [block explorer index example](https://fuellabs.github.io/fuel-indexer/master/examples/block-explorer.html).

### 2.3 Deploying our index

By now we have a brand new index that will index some blocks and transactions, but now we need to build and deploy it in order to see it in action.

#### 2.3.1 Starting an indexer service

- To start an indexer service, we'll be spinning up Postgres and Fuel indexer containers via `docker compose`. Our indexer service will connect to Fuel's `beta-2` network so that we can index blocks and transactions from an _actual_ Fuel node. We'll use the `docker compose` file below, and spinning everything up with `docker compose up`.

> IMPORTANT: Ensure that any local Postgres instance that is running on port `5432` is stopped.
>
> You can open up a `docker-compose.yaml` file in the same directory as your index manifest, and paste the YAML content below to this `docker-compose.yaml` file.

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

#### 2.3.2 Deploying your index to your Fuel indexer service

With our database and Fuel indexer indexer containers up and running, we'll deploy the index that we previously created. If all goes well, you should see the following:

```bash
forc-index deploy --manifest hello_index.manifest.yaml --url http://0.0.0.0:29987
```

```text
forc index deploy --manifest hello_index.manifest.yaml --url http://0.0.0.0:29987
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

With our index deployed, after a few seconds, we should be able to query for newly indexed data.

Below, we write a simple GraphQL query that simply returns a few fields from all transactions that we've indexed.

```bash
curl -X POST http://0.0.0.0:29987/api/graph/my_project \
   -H 'content-type: application/json' \
   -d '{"query": "query { tx { id hash status block }}", "params": "b"}' \
| json_pp
```

```text
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   364  100   287  100    77   6153   1650 --:--:-- --:--:-- --:--:--  9100
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
