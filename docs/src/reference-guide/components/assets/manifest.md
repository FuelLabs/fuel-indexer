# Manifest

A manifest serves as the YAML configuration file for a given index. A proper manifest has the following structure:

```yaml
namespace: fuel
identifier: index1
abi: path/to/my/contract-abi.json
contract_id: "0x39150017c9e38e5e280432d546fae345d6ce6d8fe4710162c2e3a95a6faff051"
graphql_schema: path/to/my/schema.graphql
start_block: 1564
module:
  wasm: path/to/my/wasm_module.wasm
report_metrics: true
```

## `namespace`

- Think of the `namespace` as an organization identifier. If you're familiar with say, [Java package naming](https://stackoverflow.com/questions/6247849/java-package-naming), then think of an index's `namespace` as being its _domain name_. The `namespace` is unique to a given index operator -- i.e., index operators will not be able to support more than one `namespace` of the same name.

## `identifier`

- The `identifier` field is used to (quite literally) identify the given index. If `namespace` is the organization/domain name, then think of `identifier` as the name of an index within that organization/domain.
- As an example, if a provided `namespace` is `"fuel"` and a provided `identifier` is `"index1"`, then the unique identifier for the given index will be `fuel.index1`.

## `abi`

- The `abi` option is used to provide a link to the Sway JSON application binary interface (JSON ABI) that is generated when you build your Sway project. This generated ABI contains all types, type IDs, and logged types used in your Sway contract.

## `contract_id`

- The `contract_id` specifies which particular contract you would like your index to subscribe to.

## `graphql_schema`

- The `graphql_schema` field contains the file path that points to the GraphQL schema for the given index. This schema file holds the structures of the data that will eventually reside in your database. You can read more about the format of the schema file [here](schema.md).

> Important: The objects defined in your GraphQL schema are called 'entities'. These entities are what will be eventually be stored in the database.

## `start_block`

- The particular start block after which you'd like your indexer to start indexing events.

## `module`

- The `module` field contains a file path that points to code that will be run as an _executor_ inside of the indexer.
- There are two available options for modules/execution: `wasm` and `native`.
  - When specifying a `wasm` module, the provided path must lead to a compiled WASM binary.

> Important: At this time, `wasm` is the preferred method of execution.

## `report_metrics`

- Whether or not to report Prometheus metrics to the Fuel backend

## `resumable`

- The resumable field of type boolean specifies whether or not the indexer should synchronise with the latest block if it has fallen out of sync. 

