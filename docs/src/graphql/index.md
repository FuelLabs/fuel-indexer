# GraphQL

The Fuel indexer uses GraphQL to in order to allow users to query for indexed data. Please note that the Fuel indexer does not support the full GraphQL specification; however, we do our best to reasonably support as much as we can. In this chapter, you can find information on how to leverage our supported features to efficiently get the data you want.

- [Directives](./directives.md)
- [GraphQL API Server](./api-server.md)
- [Playground](./playground.md)
- [Queries](../queries/index.md)

## Supported Functionality

While we do our best to maintain compliance with the GraphQL specification and parity with other implementations, there are a few things that are under development or will not be implemented. Here's a table describing our GraphQL functionality:

```text
✅ -- implemented
🚧 -- planned or in development
⛔ -- will not implement
```

| Functionality | Status | Notes |
|------|----------|-------|
| Arguments | ✅ | [read the Search and Filtering section](../queries/search-filtering.md) |
| Aliases | ✅ | |
| Fragments | ✅ | inline fragments are currently not supported |
| Introspection | ✅ | |
| GraphQL Playground | ✅ | [read the Playground section](./playground.md) |
| Pagination | ✅ | [read the Pagination section](../queries/pagination.md) |
| Directives | 🚧 | [read the Directives section](./directives.md) |
| List Types | 🚧 | |
| Union Types | 🚧 | |
| Federation | 🚧 | |
| Variables | ⛔ | |
| Mutations | ⛔ | |
| Enums | ⛔ | |
| Interfaces | ⛔ | |
| Input Types| ⛔ | |
