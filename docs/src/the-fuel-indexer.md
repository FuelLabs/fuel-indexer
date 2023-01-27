# Fuel Indexer

![Fuel Logo](./img/fuel.png)

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within a Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

By using a combination of Fuel-flavored GraphQL schema, a SQL backend, and indices written in Rust, users of the Fuel indexer can get started creating production-ready backends for their dApps, meant to go fast 🚗💨.

Feel free to check out [Quickstart](./quickstart/index.md) for those wanting to build dApp backends right away. And for those willing to contribute to the Fuel indexer project, please feel free to read [our contributor guidelines](https://github.com/FuelLabs/fuel-indexer/blob/master/docs/CONTRIBUTING.md) and the [For Contributors](for-contributors/index.md) section of the book.

## Architecture

[![](https://mermaid.ink/img/pako:eNp9kVFrwjAUhf_K5T450D9QxkCNDmEDXUUYjQ_X5lYLbdKlCVis_33pOkFBTV7CyfnuCScnTI1ijHBvqTrAWkgNYY2TuecCpsbyFkajt3bzCbEjxy1MBn9XC634yPal90-Sa7FH3ruJqw9YNxXXLYiBIEc7qvmfmd4wcLGPl4uej_O9pqJ-kPiMfu3wL_7xXLt7ySK5CNtemCXfxtvb52fGppB3CiiuCtO0zzKlvt44xJJtSbkKzZ66DInuwCVLjMJRcUa-cBKlPgcreWfiRqcYOet5iL5SoWmRU_iTEqMslMDnX4sTjJ4?type=png)](https://mermaid.live/edit#pako:eNp9kVFrwjAUhf_K5T450D9QxkCNDmEDXUUYjQ_X5lYLbdKlCVis_33pOkFBTV7CyfnuCScnTI1ijHBvqTrAWkgNYY2TuecCpsbyFkajt3bzCbEjxy1MBn9XC634yPal90-Sa7FH3ruJqw9YNxXXLYiBIEc7qvmfmd4wcLGPl4uej_O9pqJ-kPiMfu3wL_7xXLt7ySK5CNtemCXfxtvb52fGppB3CiiuCtO0zzKlvt44xJJtSbkKzZ66DInuwCVLjMJRcUa-cBKlPgcreWfiRqcYOet5iL5SoWmRU_iTEqMslMDnX4sTjJ4)

The Fuel indexer is meant to run alongside a Fuel node and a database. Generally, the typical flow of information through the indexer is as follows:

1. A Sway smart contract emits receipts during its execution on the Fuel node.
2. Blocks, transactions, and receipts from the node are monitored by the Fuel indexer service and checked for specific user-defined event types.
3. When a specific event type is found, the indexer executes the corresponding handler from an index module.
4. The handler processes the event and stores the index information in the database.
5. A dApp queries for blockchain data by using the indexer's GraphQL API endpoint, which fetches the desired information from the corresponding index in the database and returns it to the user.
