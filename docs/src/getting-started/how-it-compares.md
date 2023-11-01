# How it Compares

Since many users may be familiar with indexing by using a solution like The Graph, it may be helpful to provide a comparison between The Graph and the Fuel indexer.

## Differential Value

Generally, the biggest conceptual differences between Fuel's indexer service and other indexer services (such as The Graph) are indexing speed, general ease of use, and indexable data types.

### Speed of indexing

Using Fuel's indexers, users can index about 30 blocks per second on a standard Macbook Pro on an M1 chip. This type of indexing speed is a boon to smart contract authors who need to iterate quickly while building dApps.

### General usability

Unlike other indexing services, users can use the forc index CLI tool to create, deploy, update, re-deploy, remove, and check the status of their indexers. The ability to completely manage, maintain, and improve remote indexers without having to open any files, or edit any source code completely sets Fuel's indexer apart from other services in the space.

### What you can index

The Fuel indexer is tailored for compatibility with the FuelVM. This means that instead of being limited to the primitives of the Etherem virtual machine (EVM), users of the Fuel indexer gain access to a much richer set of indexable abstractions provided by the FuelVM (e.g. predicates, transaction receipts, etc).

Legend:

- 🟩 : Supported
- 🟥 : Not supported
- 🟨 : Planned

|  Feature |  The Graph  | Fuel Indexer   | Notes  |
|:-:|:-:|:-:|:-:|
|  Hosted Indexers |  🟩  | 🟩   |   |
|  WASM Execution | 🟩   |  🟩  |   |
|  Handlers | 🟩 | 🟩 | see [Indexing Fuel Types](../indexing-fuel-types/index.md) and [Indexing Custom Types](../indexing-custom-types/index.md)|
|  Updateable Schemas | 🟩   |  🟩  |   |
|  API Authentication | 🟩   |  🟩  |   |
|  Starting Block Configuration | 🟩   |  🟩  |   |
|  Native Unit Testing Framework |  🟩  | 🟥 | Users are able to use `cargo test` |
|  GraphQL: Sorting, Pagination, Filtering | 🟩   |  🟩  |   |
|  Schema: Enum, Object, and Union types | 🟩   |  🟩  |   |
|  Schema: One-to-one, one-to-many, many-to-many relationships | 🟩   |  🟩  |   |
|  AssemblyScript Support |  🟩 |  🟥 |   |
|  Admin Portal UI |  🟩 |  🟥 |    |
| Stop, Remove, Re-deploy indexers without smart contract changes | 🟥  |  🟩  |  |
| Update & redeploy indexers with 0 downtime |  🟥  |  🟩 |  |
| Use third party dependencies in your indexers  |  🟥  |  🟩  |  |
