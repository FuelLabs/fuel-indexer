# hello-world-native

## Usage

> NOTE: Commands are run from `fuel-indexer/examples/hello-world-native`

### Start a Fuel node

```bash
cargo run -p hello-world-node --bin hello-world-node
```

### Start the indexer service

> IMPORTANT: Remember that unlike WebAssembly (WASM) execution, native execution builds a binary
> that can be invoked using either `cargo` or executed directly.

```bash
cargo run -p hello_indexer_native --bin hello_indexer_native -- \
    --manifest examples/hello-world-native/hello-indexer-native/hello_indexer_native.manifest.yaml \
    --run-migrations
```

### Interact

Trigger some test data by simulating a contract call.

```bash
cargo run -p hello-world-data --bin hello-world-data -- --host 0.0.0.0:4000
```

### Validate

Ensure that test data was indexed via a GraphQL query:
  1. Open this GraphQL playground link http://localhost:29987/api/playground/fuel/explorer.
  2. Submit the following query

```graphql
query {
    transaction {
        id
        time
        block
        label
    }
}