# hello world

 A "Hello World" type of program for the Fuel Indexer service.

## Usage

### Spin up the test Fuel node

A small `hello-world-test-node` package is provided to spin up a short-lived Fuel node.

```bash
cargo run --bin hello-world-test-node
```

### Deploy the indexer

```bash
forc index deploy \
   --path hello-index \
   --output-dir-root /path/to/repository/fuel-indexer \
   --url http://0.0.0.0:29987 \
   --target wasm32-unknown-unknown \
   --release
```

### Interact

Trigger some test data by simulating a contract call.

```bash
cargo run --bin hello-world-data -- --host 0.0.0.0:4000
```

### Validate

Ensure that test data was indexed via a GraphQL query.

```bash
curl -X POST http://0.0.0.0:29987/api/graph/fuel_examples/hello_index \
   -H 'content-type: application/json' \
   -d '{"query": "query { salutation { id message_hash message greeter first_seen last_seen }}", "params": "b"}' \
| json_pp
```
