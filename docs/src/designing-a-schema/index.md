# Designing a Schema

The Fuel indexer uses GraphQL in order to allow users to query for indexed data. In this chapter, you can find information on how to leverage our supported features to efficiently get the data you want.

> ⚠️ Please note that the Fuel indexer does not support the full GraphQL specification; however, we do our best to reasonably support as much as we can.

- [Types](./types.md)
- [Scalars](./scalars.md)
- [Directives](./directives.md)
- [Relationships](./relationships.md)

## Supported Functionality

While we do our best to maintain compliance with the GraphQL specification and parity with other implementations, there are a few things that are under development or will not be implemented. Here's a table describing our GraphQL functionality:

Legend:

- 🟩 : Functionally complete
- 🟨 : Partially complete
- 🟥 : Planned but incomplete
- ⛔ : Not planned

| Functionality | Status | Notes |
|------|----------|-------|
| Arguments | 🟩 | [read the Search and Filtering section](../querying/search-and-filtering.md) |
| Aliases | 🟩 | |
| Fragments | 🟨 | inline fragments are currently not supported |
| Introspection | 🟩 | |
| GraphQL Playground | 🟩 | [read the Playground section](../querying/playground.md) |
| Pagination | 🟨 | [read the Pagination section](../querying/pagination.md) |
| Directives |🟨 | [read the Directives section](./directives.md) |
| List Types |🟨 | |
| Union Types |🟨 | |
| Federation |⛔ | |
| Variables | ⛔ | |
| Mutations | ⛔ | |
| Enums | 🟨 | |
| Interfaces | ⛔ | |
| Input Types| ⛔ | |
