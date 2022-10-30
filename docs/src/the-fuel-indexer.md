# Fuel Indexer

![Fuel Logo](./img/fuel.png)

The Fuel indexer is a standalone service that can be used to index various components of the blockchain. These indexable components include blocks, transactions, receipts, and state within a Fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

By using a combination of Fuel-flavored GraphQL schema, a SQL backend, and indices writting in Rust (that compile to WASM), users of the Fuel indexer can get started creating production-ready backends for their dApps, meant to go blazing fast 🚗💨.

Feel free to checkout [Getting Started](./getting-started/index.md) for those wanting to build dApp backends right away. And for those willing to contribute to the Fuel indexer project, please feel free to read [our contributor guidelines](https://github.com/FuelLabs/fuel-indexer/blob/master/docs/CONTRIBUTING.md).
