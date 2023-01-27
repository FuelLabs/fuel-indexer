# Fuel Indexer

![Fuel Logo](./img/fuel.png)

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within a Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

By using a combination of Fuel-flavored GraphQL schema, a SQL backend, and indices written in Rust, users of the Fuel indexer can get started creating production-ready backends for their dApps, meant to go fast 🚗💨.

Feel free to check out [Quickstart](./quickstart/index.md) for those wanting to build dApp backends right away. And for those willing to contribute to the Fuel indexer project, please feel free to read [our contributor guidelines](https://github.com/FuelLabs/fuel-indexer/blob/master/docs/CONTRIBUTING.md) and the [For Contributors](for-contributors/index.md) section of the book.

## Architecture

[![](https://mermaid.ink/img/pako:eNp9kUGLwjAQhf_KMCcX9A8UWVCjIuxhXb0sjYexmdZCm9Q0gS3W_76pVVBQM5fw8d688HLCxCjGCDNL1QG2QmoIZxIvPBcwM5Z3MBp9thtHjluYDi58pRX_sf3oxdP4Hvb6Zbdu_QXbpuK6BTEQ5GhPNV89swcP3OST79U1L880FfWLxHfucWf_4aPn2j1LFvEN7Howj3-Nt4_PT41NIO8IKK4K07TvMqW-HxxiybakXIVaT12GRHfgkiVG4ao4JV84iVKfg5S8M5tGJxg563mIvlKhaZFT-JASozSUwOd_QceL2w?type=png)](https://mermaid.live/edit#pako:eNp9kUGLwjAQhf_KMCcX9A8UWVCjIuxhXb0sjYexmdZCm9Q0gS3W_76pVVBQM5fw8d688HLCxCjGCDNL1QG2QmoIZxIvPBcwM5Z3MBp9thtHjluYDi58pRX_sf3oxdP4Hvb6Zbdu_QXbpuK6BTEQ5GhPNV89swcP3OST79U1L880FfWLxHfucWf_4aPn2j1LFvEN7Howj3-Nt4_PT41NIO8IKK4K07TvMqW-HxxiybakXIVaT12GRHfgkiVG4ao4JV84iVKfg5S8M5tGJxg563mIvlKhaZFT-JASozSUwOd_QceL2w

The Fuel indexer is meant to run alongside a Fuel node and a database. Generally, the typical flow of information through the indexer is as follows:

1. A Sway smart contract emits receipts during its execution on the Fuel node.
2. Blocks, transactions, and receipts from the node are monitored by the Fuel indexer service and checked for specific user-defined event types.
3. When a specific event type is found, the indexer executes the corresponding handler from an index module.
4. The handler processes the event and stores the index information in the database.
5. A dApp queries for blockchain data by using the indexer's GraphQL API endpoint, which fetches the desired information from the corresponding index in the database and returns it to the user.
