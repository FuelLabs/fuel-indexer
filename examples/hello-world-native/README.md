# hello-world-native

## Usage

### Start a Fuel node

```bash
cargo run --bin hello-world-node
```

### Start the indexer service

```bash
cargo run --bin hello_indexer_native -- \
    --manifest examples/hello-world-native/hello-indexer-native/hello_indexer_native.manifest.yaml
    --run-migrations
```

### Trigger some data

```bash
cargo run --bin hello-world-data
```

### Query some data

```bash
curl -X POST http://0.0.0.0:29987/api/graph/fuel_examples/hello_indexer_native \
   -H 'content-type: application/json' \
   -d '{"query": "query { salutation { id message_hash message greeter first_seen last_seen }}", "params": "b"}' \
| json_pp
```