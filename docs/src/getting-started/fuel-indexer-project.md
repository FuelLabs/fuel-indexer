# A Fuel Indexer Project

The Fuel indexer service can currently be used in two different ways:

1. The service can be run as a standalone binary, outside the scope of a larger Fuel project.
2. The service can be included in a larger Fuel project, as a tandem service.

We'll describe these two different implementations below.

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
        ├── hello-index.manifest.yaml
        ├── schema
        │   └── hello-index.schema.graphql
        └── src
            └── lib.rs

8 directories, 7 files
```
