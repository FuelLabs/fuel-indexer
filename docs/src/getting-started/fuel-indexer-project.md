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
➜  my-project tree . -I target/ -I out/
.
├── contracts
│   └── my-contract
│       ├── Forc.lock
│       ├── Forc.toml
│       ├── out
│       │   └── debug
│       │       ├── my-contract-abi.json
│       │       ├── my-contract-storage_slots.json
│       │       └── my-contract.bin
│       ├── src
│       │   └── main.sw
│       └── tests
│           └── harness.rs
├── frontend
│   └── index.html
└── indexer
    ├── my-index.manifest.yaml
    ├── my-index
    │   ├── Cargo.toml
    │   └── src
    │       └── lib.rs
    └── schema
        └── schema.graphql

11 directories, 14 files
```
