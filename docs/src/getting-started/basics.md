# Basic usage of the Fuel Indexer

## Database

The indexer will need a database backend, currently we support postgres. You may bring up an instance with docker, if you like:

`docker run --rm -p 5432:5432 --name postgres-db -e POSTGRES_PASSWORD=my-secret -d postgres`

## Indexer Service config

A basic indexer service can be configured with a yaml file, like so:

```yaml
---
fuel_node_addr: "127.0.0.1:30333"
database_url: "postgres://postgres:my-secret@127.0.0.1:5432"
listen_endpoint: "127.0.0.1:29899"
```

- The `fuel_node_addr` will point to your fuel node. You can optionally supply the `--local` flag to the service and the indexer will bring up an embedded fuel node, useful for dev purposes to bring up all components on your local machine. This field will then be ignored in that case.
- `database_url` will point at your database instance
- `listen_endpoint` the port the API server will listen on

## Instantiating a Fuel Indexer

Start the Fuel Indexer service with a reference to your Fuel node

```bash
RUST_LOG=wasmer_compiler_cranelift=off,regalloc=off,info cargo r --bin service configs/local_config.yaml --local --test-manifest manifest.yaml
```

- `wasmer_compiler_cranelift` and `regalloc` are both quite noisy at the info level, so turn them on only if you need to!
- The format of `local_config.yaml` was described above. We'll get to `manifest.yaml` in the next section.
- Additionally, note the `--local` flag, this will automatically bring up a fuel-core node embedded into the service. If you would prefer to bring one up yourself, be sure to set the `fuel_node_addr` in the manifest.
- If it all comes up properly, you'll see a message like this:

```console
INFO fuel_core::service::graph_api: Binding GraphQL provider to 127.0.0.1:33641
INFO fuel_wasm_executor::service: Registered indexer test_namespace
```

## Starting an API server

With the same config.yaml as above, run the following command:

```bash
cargo r --bin api_server -- --config configs/local_config.yaml
```

You should be up and running and ready to try out an indexer module! We'll talk about that next.
