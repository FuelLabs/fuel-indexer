# Manifest

A manifest serves as the configuration for your indexer and it is written in YAML format. A proper manifest has the following structure:

```yaml
---
namespace: ...
graphql_schema: ...
identifier: ...
module:
  wasm:
    ...
handlers:
  - event: ...
    handler: ...
```

## Namespace

Your graph will live within the namespace set as part of this field. As such, it's important that you remember it when querying the API endpoint for your data. For example, if the field is set to `my_special_space`, then your queries would look similar to this:

```sh
curl -s localhost:29987/api/graph/my_special_space -XPOST -H ...
```

## GraphQL Schema

The `graphql_schema` field contains the file path to a GraphQL schema. This schema holds the structures of the data that will eventually reside in your database. You can read more about the format of the schema file [here](schema.md).

## Identifier

The `identifier` field serves to differentiate indexes that are part of the same namespace.

## Module

The `module` field contains a file path to custom code that will be executed as part of the indexer. There are two available options: `wasm` and `native`. If you choose to use WASM, the path must lead to a compiled WASM module. Alternatively, if you choose to use the native option, the path must lead to a module that contains native Rust code. In both cases, the functions included as part of the `handlers` field should be present in the module.

## Handlers

The `handlers` field maps event types to the names of function that will handle each event. The event should map to an input type that is present in a contract's ABI specification.
