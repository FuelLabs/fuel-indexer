# Assets

An indexer is constructed of a few assets: a manifest file, a GraphQL schema file, and a WASM module.

- [Manifest](./manifest.md)
  - Supplies metadata regarding how a given indexer should be built and run.
- [Schema](./schema.md)
  - A GraphQL schema file that defines your index data model via GraphQL types.
- [Module](./module.md)
  - An executor that gets registered into a Fuel indexer at runtime.
