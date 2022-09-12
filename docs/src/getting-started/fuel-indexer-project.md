# A Fuel Indexer Project

The Fuel Indexer service can currently be used in two different ways:

1. The service can either be run as a standalone binary, outside the scope of a Sway project.
2. The service can be included in a Sway project, as a tandem service.

We'll describe these two different implementations below.

## As a standalone service

When running a Fuel Indexer service as a standalone binary, you can just start the service (with or without Docker) after you've defined your manifest file. For an example of this, checkout the [Hello World example](./../examples/hello-indexer.md) section.

## With a Sway project

The convetion for a Sway project layout including a Fuel Indexer is:

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
    ├── contracts
    │   └── my-contract-abi.json
    ├── my-index.manifest.yaml
    ├── my-indexer
    │   ├── Cargo.toml
    │   └── src
    │       ├── my-index.rs
    │       └── my-second-index.rs
    ├── my-second-index.manifest.yaml
    └── schema
        └── schema.graphql

12 directories, 15 files
```
