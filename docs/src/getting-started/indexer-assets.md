# Indexer Assets

There are three primary assets used to build a Fuel Indexer index.

- Manifest Files
  - YAML files that include details for one or many indices.
- WASM Modules
  - WebAssembly blobs compiled for `wasm32-unknown-unknown` targets.
- Schema Files
  - Typical GraphQL schema files that use [Fuel Indexer types](../components/database/types.md).

## Manifest Files

The Fuel Indexer service can either run a single index or _many_ indices. Each index used by the Fuel Indexer services requires a manifest configuration that specifies index details such as: namespace, GraphQL schema location, WASM module location.

It's important to remember that a manifest file may contain details for a single index:

```yaml
namespace: my_namespace
graphql_schema: schema/schema.graphql
wasm_module: wasm_module_one.wasm
handlers:
  - event: LogData
    handler: function_one
```

or, details for several indices


```yaml
---
namespace: my_namespace
graphql_schema: schema/schema.graphql
wasm_module: wasm_module_one.wasm
handlers:
  - event: LogData
    handler: function_one
---
namespace: my_namespace
graphql_schema: schema/schema.graphql
wasm_module: wasm_module_two.wasm
handlers:
  - event: LogData
    handler: function_two
```

When a Fuel Indexer process starts, the assets at these paths, specified in the given manifest, are loaded into the service and used until the service is stopped.

## WASM Modules

WASM modules are blobs compiled for `wasm32-unknown-unknown` targets, that contain all of the functionality needed to make your index work in an isolated environemnt -- the WASM runtime. Once you've compiled your library down to WASM you can upload this new module to the indexer to see your index's changes quickly applied.

## Schema Files

As aforementioned, GraphQL schema file assets are just typical GraphQL schema files that use [Fuel Indexer types](../components/database/types.md).
