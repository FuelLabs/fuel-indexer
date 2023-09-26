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

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within the Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

<font size="4">Want to get started right away? Check out our [Quickstart](https://docs.fuel.network/docs/indexer/getting-started/quickstart/)!</font>

### Write your contract

Write your Sway contract.

```sway
struct UserAccount {
  name: str[255],
  owner: Identity
}

impl Sway for Contract {
  fn create_account(name: str[255], owner: Identity) -> Account {
    Account {
      name: "0000000000000000000000000000000000000000000000000000000000000000",
      owner: Identity::Address(Address::new(0000000000000000000000000000000000000000000000000000000000000000))
    } 
  }
}
```

### Create your project

Create your project.

```graphql
forc index new my-indexer --namespace fuellabs
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

### Write your indexing module

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