<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD041 -->
<p align="center">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="./img/fuel-indexer-logo-dark.png">
        <img alt="Fuel Indexer logo" width="400px" src="./img/fuel-indexer-logo-light.png">
    </picture>

</p>
<p align="center">
    <a href="https://github.com/FuelLabs/fuel-indexer/actions/workflows/ci.yml" alt="CI">
        <img src="https://img.shields.io/github/actions/workflow/status/FuelLabs/fuel-indexer/ci.yml?event=release" />
    </a>
    <a href="https://docs.rs/fuel-indexer/" alt="docs.rs">
      <img src="https://docs.rs/fuel-indexer/badge.svg" />
    </a>
    <a href="https://crates.io/crates/fuel-indexer" alt="crates.io">
        <img src="https://img.shields.io/crates/v/fuel-indexer?label=latest" />
    </a>
    <a href="https://crates.io/crates/fuel-indexer" alt="img-shields">
      <img alt="GitHub commits since latest release (by date including pre-releases)" src="https://img.shields.io/github/commits-since/FuelLabs/fuel-indexer/latest?include_prereleases">
    </a>
    <a href="https://discord.gg/xfpK4Pe" alt="Discord">
      <img src="https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2" />
    </a>
</p>

## What is the Fuel indexer?

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within the Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

> TLDR: It's Infrastructure as a service (IaaS) for Web3 dApp backends.


**<u>Want to get started right away? Check out our [Quickstart!](https://docs.fuel.network/docs/indexer/getting-started/quickstart/)</u>**

## Install

Fuel's indexer supports Linux (x64 & arm64) and macOS (x64 & Apple Silicon).

> If you don't want to deal with dependency issues we recommend just using Fuel's indexer via Docker, [via the included docker-compose file](https://github.com/FuelLabs/fuel-indexer/blob/develop/scripts/docker-compose.yaml).
>
> Otherwise, users can find the `fuel-indexer` binary installed via fuelup.

Install Fuel's toolchain manager - fuelup.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://install.fuel.network/fuelup-init.sh | sh
```

## Usage

For development, users will primarily use the `forc index` command line utility made available after installing fuelup.

```bash
forc index --help
```

```text
forc index 0.0.0
Fuel Indexer Orchestrator

USAGE:
    forc-index <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    auth        Authenticate against an indexer service
    build       Build an indexer
    check       Check for Fuel indexer components
    deploy      Deploy an indexer to an indexer service
    help        Print this message or the help of the given subcommand(s)
    kill        Kill the indexer process. Note that this command will kill any process listening
                    on the default indexer port or the port specified by the `--port` flag
    new         Create a new indexer project in a new directory
    postgres    Fuel Postgres Orchestrator
    remove      Stop and remove a running indexer
    start       Standalone binary for the Fuel indexer service
    status      Check the status of a registered indexer

```

<!--

### Write your contract

Write your Sway contract.

```sway
struct UserAccount {
  name: str[255],
  owner: Identity
}

abi MyContract {
  fn create_account(name: str[255], owner: Identity) -> Account;
}

impl MyContract for Contract {
  fn create_account(name: str[255], owner: Identity) -> Account {
    Account {
      name: "0000000000000000000000000000000000000000000000000000000000000000",
      owner: Identity::Address(Address::from(0000000000000000000000000000000000000000000000000000000000000000))
    } 
  }
}
```

### Define your data model

Define your data model using GraphQL schema.

```graphql
type Account @entity {
  id: ID!
  name: Charfield!
  address: Address!
}
```

### Write your indexer module

Define your handler logic in your indexer module.

```rust
use fuel_indexer_utils::*;

mod indexer_mod {
    fn handle_account(account: Account) {
      let account = FuelAccount::new(account.name, account.address).get_or_create();
      info!("Account address is: {}", account.address);
    }
}
```

### Query your data

Query your data using GraphQL.

```graphql
query {
  account {
    name
    owner
  }
}
```

For more info, [checkout the docs](https://docs.fuel.network/docs/indexer/), or checkout the [Quickstart](https://docs.fuel.network/docs/indexer/getting-started/quickstart/)!

-- >