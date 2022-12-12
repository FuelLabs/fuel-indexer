# A Fuel Indexer Project

The Fuel indexer project can currently be used in two different ways:

1. Indexer tooling can be used to compile arbitrary indicies.
2. The indexer service can be run as a standalone binary, outside the scope of a larger Fuel project.
3. The indexer service can be included in a larger Fuel project, as a tandem service.

We'll describe these three different implementations below.

## Compiling arbitrary indices

For info on how to use indexer tooling to compile arbitrary indices, checkout our [Quickstart](./../quickstart/index.md)

## As a standalone service

When running a Fuel indexer service as a standalone binary, you can just simply start the service after migrations have been run.

## With a Fuel project

The convetion for a Fuel project layout including a Fuel indexer is:

```text
.
├── contracts
│   └── greeting
│       ├── Forc.toml
│       └── src
│           └── main.sw
├── frontend
│   └── index.html
└── indexer
    └── hello-index
        ├── Cargo.toml
        ├── hello_index.manifest.yaml
        ├── schema
        │   └── hello_index.schema.graphql
        └── src
            └── lib.rs

8 directories, 7 files
```
