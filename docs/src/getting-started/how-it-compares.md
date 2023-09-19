# How it Compares

Since many users may be familiar with indexing by using a solution like The Graph, it may be helpful to provide a comparison between it and the Fuel indexer. Generally, the biggest conceptual differences between the two are as follows:

- The Fuel indexer does not support publishing indexers to decentralized networks. That being said, the Fuel indexer is developed with the intention of allowing anyone to run an instance and host whatever indexers they want.
- Contracts on the Fuel network can be indexed without the need to add additional events. Whether a user wanted to index information from the very start of a contract or a recent block, the process is the same: define the schema and manifest, write custom handler logic, and deploy to an indexer service.

Legend:

- 🟩 : Functionally complete
- 🟨 : Partially complete
- 🟥 : Planned but incomplete
- ⛔ : Not planned

<!-- todo: continue this from https://www.notion.so/fuellabs/Indexer-Feature-Matrix-24d654f1bc3d411e8371d13db90fe237 -->

|  Feature |  The Graph  | Fuel Indexer   | Notes  |
|:-:|:-:|:-:|:-:|
|  Hosted Indexers |  🟩  | 🟩   |   |
|  WASM Execution | 🟩   |  🟩  |   |
|  Native Execution | 🟥   |  🟩  |   |
|  Handlers | 🟩 | 🟩 | see [Indexing Fuel Types](../indexing-fuel-types/index.md) and [Indexing Custom Types](../indexing-custom-types/index.md)|
|  Updateable Schemas | 🟩   |  🟩  |   |
|  API Authentication | 🟩   |  🟩  |   |
|  Starting Block Configuration | 🟩   |  🟩  |   |
|  Grafted Indexes |  🟩  | 🟥 |  |
|  Index Ownership Transfer |  🟩  | 🟥 |  |
|  Unit Testing Framework |  🟩  | 🟥 | Users are able to use `cargo test` |
|  Querying: Sorting | 🟩   |  🟩  |   |
|  Querying: Pagination | 🟩   |  🟩  |   |
|  Querying: Filtering | 🟩   |  🟩  |   |
|  Querying: Time Travel |  🟩  | 🟥 |  |
|  Querying: Fulltext Search |  🟩  | 🟥 |  |
|  Querying: GraphQL Validation Spec |  🟩  | 🟨 |  |
|  Schema: Enums | 🟩   |  🟩  |   |
|  Schema: One-to-one relationships | 🟩   |  🟩  |   |
|  Schema: One-to-many relationships| 🟩   |  🟩  |   |
|  Schema: Many-to-many relationships | 🟩   |  🟩  |   |
|  Schema: Reverse Lookup |  🟩  | 🟥 |  |
|  AssemblyScript Support |  🟩 |  ⛔ |   |
|  Admin Portal UI |  🟩 |  ⛔ |   |
|  Decentralized Hosting |  🟩 |  ⛔ |   |
