# What Can I Index?

You can index three main types of data from the Fuel network: blocks, transactions, and transaction receipts. You can read more about these data types below:

- [**Blocks and Transactions**](./blocks-and-transactions.md)

- [**Transaction Receipts**](./receipts.md)

If you've previously built an indexer for the EVM, you may be used to only being able to index data that is emitted as an event.

However, with Fuel you can index the entire transaction, which means you can use much more than logged data, allowing you to reduce the number of logs you need in your contract.
