# A Fuel Indexer Project

The Fuel Indexer service can currently be used in two different ways:

1. The service can either be run as a standalone binary, outside the scope of a Sway project.
2. The service can be included in a Sway project, and as a tandem service.

We'll describe these two different implementations below.

## As a standalone service

When running a Fuel Indexer service as a standalone binary, you can just start the service (with or without Docker) after you've defined your manifest file. For an example of this, checkout the [Hello World example](./../examples/hello-indexer.md) section.

## With a Sway project

When running a Fuel Indexer service as a tandem service inside of a larger Sway project, the convention is to typically include your indexer binary (`main.rs`) inside an `indexer/` folder in the root of the project (`my-project`). From here, you can easily run your indexer with WASM execution, or non-WASM execution (or both!).

The convetion for a Sway project layout including a Fuel Indexer is:

```text
➜  my-project tree .
.
├── Forc.toml
├── contracts
│   └── main.sw
├── frontend
│   └── index.html
├── indexer
│   ├── contracts
│   │   └── my-project-abi.json
│   ├── lib.rs
│   ├── main.rs
│   ├── manifest.yaml
│   └── schema
│       └── my-project-schema.graphql
└── tests
    └── harness.rs

6 directories, 9 files
```
