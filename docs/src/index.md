<!-- markdownlint-disable MD033 -->
# 🗃 Fuel Indexer

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within the Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

By using a combination of Fuel-flavored GraphQL schema, a SQL backend, and indexers written in Rust, users of the Fuel indexer can get started creating production-ready backends for their dApps, meant to go fast 🚗💨.

For those wanting to build dApp backends right away, feel free to check out the [Quickstart](./getting-started/quickstart.md). And for those willing to contribute to the Fuel indexer project, please feel free to read [our contributor guidelines](https://github.com/FuelLabs/fuel-indexer/blob/develop/docs/CONTRIBUTING.md) as well as the [For Contributors](for-contributors/index.md) chapter of the book.
<!-- KEEP THIS IN CASE WE WANT TO MAKE DIAGRAM CHANGES [![fo](https://mermaid.ink/img/pako:eNp9kWFrwjAQhv_KcZ8U9A-UIajZhrDBZkUYjR9uzamFNqlpAhbrf1-6TFCY5r6EN-9zb7g7YW4UY4I7S_UeVkJqCGeavXguYW4sb2A8nnTrd0gdOe5gNvh9WmjFR7bD6J9l12JEXvuOn2-wamtuOhADQY6-qeE_Zn7DwMU-_VhEPi12msrmTuIjGp56fskHz437L1pkF2EThefsy3h7-_-tsTkUvQKK69K03aNQqa8LR1ixrahQYbSnPkOi23PFEpNwVbwlXzqJUp-DlbwzaatzTJz1PEJfqzBqUVBYShXF8w9dz4xz?type=png)](https://mermaid.live/edit#pako:eNp9kWFrwjAQhv_KcZ8U9A-UIajZhrDBZkUYjR9uzamFNqlpAhbrf1-6TFCY5r6EN-9zb7g7YW4UY4I7S_UeVkJqCGeavXguYW4sb2A8nnTrd0gdOe5gNvh9WmjFR7bD6J9l12JEXvuOn2-wamtuOhADQY6-qeE_Zn7DwMU-_VhEPi12msrmTuIjGp56fskHz437L1pkF2EThefsy3h7-_-tsTkUvQKK69K03aNQqa8LR1ixrahQYbSnPkOi23PFEpNwVbwlXzqJUp-DlbwzaatzTJz1PEJfqzBqUVBYShXF8w9dz4xz) -->

## Architecture

<!-- Using an <img> so we can size it -->
<img src="https://i.imgur.com/8K14p9h.png" alt="diagram" width="500"/>

The Fuel indexer is meant to run alongside a Fuel node and a database. Generally, the typical flow of information through the indexer is as follows:

1. A Sway smart contract emits receipts during its execution on the Fuel node.
2. Blocks, transactions, and receipts from the node are monitored by the Fuel indexer service and checked for specific user-defined event types.
3. When a specific event type is found, an indexer executes the corresponding handler from its module.
4. The handler processes the event and stores the indexed information in the database.
5. A dApp queries for blockchain data by using the indexer's GraphQL API endpoint, which fetches the desired information from the corresponding index in the database and returns it to the user.
