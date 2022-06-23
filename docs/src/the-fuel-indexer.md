# Fuel Indexer

![Fuel Logo](./img/fuel.png)

The Fuel Indexer is a standalone binary that can be used to index various components of [the blockchain]. These indexable components include blocks, transactions, and [receipts](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/tx_format.md) and state within a fuel network, allowing for high-performance read-only access to the blockchain for advanced dApp use-cases.

Events can be indexed by the Fuel Indexer by either using WASM modules (as described in [the Hello World example](./src/examples/hello-indexer.md)), or by writing native Rust handlers (as described in [the Simple Non-WASM example]())
