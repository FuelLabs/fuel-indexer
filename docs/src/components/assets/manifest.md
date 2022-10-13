# Manifest

A manifest serves as the YAML configuration file for a given index. A proper manifest has the following structure:

```yaml
namespace: fuel_indexer_test
graphql_schema: fuel-indexer-tests/assets/fuel_indexer_test.graphql
identifier: index1
module:
  wasm: fuel-indexer-tests/assets/fuel_indexer_test.wasm
handlers:
  - event: LogData
    handler: function_one
```

## `namespace`

- Think of the `namespace` as an organization identifier. If you're familiar with say, [Java package naming](https://stackoverflow.com/questions/6247849/java-package-naming), then think of an index's `namespace` as being its _domain name_. The `namespace` is unique to a given index operator -- i.e., index operators will not be able to support more than one `namespace` of the same name.

## `graphql_schema`

- The `graphql_schema` field contains the file path that points to the GraphQL schema for the given index. This schema file holds the structures of the data that will eventually reside in your database. You can read more about the format of the schema file [here](schema.md).

> Important: The objects defined in your GraphQL schema are called 'entities'. These entities are what will be eventually be stored in the database.

## `identifier`

- The `identifier` field is used to (quite literally) identify the given index. If `namespace` is the organization/domain name, then think of `identifier` as the name of an index within that organization/domain.
- As an example, if a provided `namespace` is `"fuel"` and a provided `identifier` is `"index1"`, then the unique identifier for the given index will be `fuel.index1`.

## `module`

- The `module` field contains a file path that points to code that will be run as an _executor_ inside of the indexer.
- There are two available options for modules/execution: `wasm` and `native`.
  - When specifying a `wasm` module, the provided path must lead to a compiled WASM binary.

> Important: At this time, `wasm` is the preferred method of execution.
