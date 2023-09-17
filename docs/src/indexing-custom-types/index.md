# Custom Types

In addition to Fuel-specific types, you can also index custom Sway types as well. To index custom types from a Sway smart contract, you'll need that contract's ABI in JSON format; the JSON ABI is generated as a result of running `forc build` to build your contract. After that, the process is similar to [indexing Fuel types](../indexing-fuel-types/index.md).

Let's go over an example.

## Contract

First, let's create a Sway contract with some simple types.

```sway
contract;

use std::logging::log;

struct Addition {
    added_value: u64,
    updated_total: u64,
}

struct Subtraction {
    subtracted_value: u64,
    updated_total: u64,
}

abi ValueStore {
    #[storage(read, write)]
    fn add(value: u64);
    
    #[storage(read, write)]
    fn subtract(value: u64) -> Subtraction;
}

storage {
    total: u64 = 1000,
}

impl ValueStore for Contract {
    #[storage(read, write)]
    fn add(value: u64) {
        let updated_total = storage.total.read() + value;
        storage.total.write(updated_total);
        log(
            Addition {
                added_value: value,
                updated_total
            }
        )
    }

    #[storage(read, write)]
    fn subtract(value: u64) -> Subtraction {
        let updated_total = storage.total.read() - value;
        storage.total.write(updated_total);

        Subtraction {
            subtracted_value: value,
            updated_total
        }
    }
}
```

In this contract, we have two types: `Addition` and `Subtraction`. As we'll soon see, indexers can process custom types that are logged or returned as part of a function. To begin creating an indexer for this contract, let's build the contract and generate a JSON ABI file. Running `forc build` generates a JSON ABI similar to the lightly-edited one below:

```json
{
  "types": [
    {
      "typeId": 0,
      "type": "()",
      "components": [],
      "typeParameters": null
    },
    {
      "typeId": 1,
      "type": "struct Addition",
      "components": [
        {
          "name": "added_value",
          "type": 3,
          "typeArguments": null
        },
        {
          "name": "updated_total",
          "type": 3,
          "typeArguments": null
        }
      ],
      "typeParameters": null
    },
    {
      "typeId": 2,
      "type": "struct Subtraction",
      "components": [
        {
          "name": "subtracted_value",
          "type": 3,
          "typeArguments": null
        },
        {
          "name": "updated_total",
          "type": 3,
          "typeArguments": null
        }
      ],
      "typeParameters": null
    },
    {
      "typeId": 3,
      "type": "u64",
      "components": null,
      "typeParameters": null
    }
  ],
  "functions": [...],
  "loggedTypes": [
    {
      "logId": 0,
      "loggedType": {
        "name": "",
        "type": 1,
        "typeArguments": []
      }
    }
  ],
  "messagesTypes": [...],
  "configurables": [...]
}

```

## Schema

To index the contracts and store information about our Sway types in the database, we should create a schema. Let's design a schema that has an entity for each Sway type:

```graphql
type AddEntity @entity {
  id: ID!
  value: UInt8!
  updated_total: UInt8!
}

type SubtractEntity @entity {
  id: ID!
  value: UInt8!
  updated_total: UInt8!
}
```

## Manifest

Before writing the handler code for the types, we need to make sure that our indexer manifest contains the necessary information to allow for the compiler to parse our contract types. Specifically, we should ensure that the `contract_abi` and `graphql_schema` fields point to the correct locations, respectively.

```yaml
# A namespace is a logical grouping of declared names. Think of the namespace
# as an organization identifier
namespace: fuellabs

# The identifier field is used to identify the given index.
identifier: custom_types_example

# The abi option is used to provide a link to the Sway JSON ABI that is generated when you
# build your project.
abi: path/to/custom/type/example/contract-abi.json

# The particular start block after which you'd like your indexer to start indexing events.
start_block: ~

# The particular end block after which you'd like your indexer to stop indexing events.
end_block: ~

# The `fuel_client` denotes the address (host, port combination) of the running Fuel client
# that you would like your indexer to index events from. In order to use this per-indexer
# `fuel_client` option, the indexer service at which your indexer is deployed will have to run
# with the `--indexer_net_config` option.
fuel_client: ~

# The contract_id specifies which particular contract you would like your index to subscribe to.
contract_id: ~

# The graphql_schema field contains the file path that points to the GraphQL schema for the
# given index.
graphql_schema: path/to/custom/type/example/indexer.schema.graphql

# The module field contains a file path that points to code that will be run as an executor inside
# of the indexer.
# Important: At this time, wasm is the preferred method of execution.
module:
  wasm: ~

# The resumable field contains a boolean that specifies whether or not the indexer should, synchronise
# with the latest block if it has fallen out of sync.
resumable: true
```

## Handler Logic

Finally, we can create handlers to index these particular types and store them in the database. Let's look at the following example:

```rust, ignore
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn index_addition(addition_event: Addition) {
        let addition = AddEntity {
          id: 123,
          value: addition_event.added_value,
          updated_total: addition_event.updated_total
        };
        addition.save();
    }

    fn index_subtraction(subtraction_event: Subtraction) {
        let subtraction = SubtractEntity {
          id: 123,
          value: subtraction_event.subtracted_value,
          updated_total: subtraction_event.updated_total
        };
        subtraction.save();
    }
}
```

Regardless of whether a custom type was logged (e.g. `Addition`) or returned (e.g. `Subtraction`), the type will be available for you to use in your functions. Just include the type(s) you want your function to use in the parameters, and the function will be executed whenever each of the parameters have been satisfied by an instance of the type(s).
