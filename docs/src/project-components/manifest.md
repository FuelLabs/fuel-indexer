# Manifest

A manifest is a YAML configuration file that specifies various aspects of how an indexer should function: Where should the indexer start? Where should the indexer end? What contract should the indexer subscribe to?

Below is a sample indexer manifest file

```yaml
namespace: fuellabs
identifier: order_book_v1
fuel_client: beta-4.fuel.network:80
abi: path/to/my/contract-abi.json
contract_id: "fuels0x39150017c9e38e5e280432d546fae345d6ce6d8fe4710162c2e3a95a6faff051"
graphql_schema: path/to/my/schema.graphql
start_block: 1564
end_block: 310000
module:
  wasm: path/to/my/wasm_module.wasm
```

## `namespace`

_Required._

The `namespace` is the topmost organizational level of an indexer. You can think of different namespaces as separate and distinct collections comprised of indexers. A namespace is unique to a given indexer operator -- i.e., indexer operators will not be able to support more than one namespace of the same name.

## `identifier`

_Required._

The `identifier` field is used to (quite literally) identify the given indexer. If a namespace describes a collection of indexers, then an identifier describes a unique indexer inside that collection. As an example, if a provided `namespace` is `"fuel"` and a provided `identifier` is `"index1"`, then the full identifier for the given indexer will be `fuel.index1`.

## `fuel_client`

_Optional._

The `fuel_client` denotes the address (host, port combination) of the running Fuel client that you would like your indexer to index events from. In order to use this per-indexer `fuel_client` option, the indexer service at which your indexer is deployed will have to run with the `--indexer_net_config` option.

## `abi`

_Optional._

The `abi` option is used to provide a link to the Sway JSON application binary interface (ABI) that is generated when you build your Sway project. This generated ABI contains all types, type IDs, logged types, and message types used in your Sway contract.

## `contract_id`

_Optional._

The `contract_id` specifies the particular contract to which you would like an indexer to subscribe. Setting this field to an empty string will index events from any contract that is currently executing on the network. This field accepts either a single string, or a list of strings. The indexer will index events from all IDs if a list is passed.

> Important: Contract IDs are unique to the content of a contract. If you are subscribing to a certain contract and then the contract itself is changed or updated, you will need to change the `contract_id` field of the manifest to the new ID.
> Note: This parameter supports both Bech32 contract IDs and non-Bech32 contract IDs

## `graphql_schema`

_Required._

The `graphql_schema` field contains the file path pointing to the corresponding GraphQL schema for a given indexer. This schema file holds the structures of the data that will eventually reside in your database. You can read more about the format of the schema file [here](./schema.md).

> Important: The objects defined in your GraphQL schema are called 'entities'. These entities are what will be eventually be stored in the database.

## `start_block`

_Optional._

The `start_block` field indicates the block height after which you'd like your indexer to start indexing events.

## `end_block`

_Optional._

The `end_block` field indicates the block height after which the indexer should stop indexing blocks.

> Important: If no `end_block` is added the indexer will keep listening to new blocks indefinitely.

## `module`

_Required._

The `module` field contains a file path that points to code that will be run as an _executor_ inside of the indexer.

## `resumable`

_Optional._

The `resumable` field contains a boolean value and specifies whether the indexer should synchronise with the latest block if it has fallen out of sync.
