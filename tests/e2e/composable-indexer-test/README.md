# Composable Test Indexer Project

- An end-to-end test of the Fuel Indexer service.
- Uses an ephemeral Fuel Client `fuel-node` who's wallet is saved to `wallet.json` and then loaded into the Web API `web-api` service.
- A call is made into the `web-api` service, which calls the contract deployed on the Fuel node
- The Fuel node emits a `Receipt` which is picked up by the subscribing Fuel Indexer `fuel-indexer` service and indexed into the Postgres database

## Testing steps

### 1. Get the latest Postgres image

```bash
docker pull postgres:latest
```

### 2. Build the example indexer app

```bash
cd tests/e2e/composable-indexer-test/indexer

cargo build --release
```

### 3. Add a symlink to the release binary

> For the sake of this test, we are ensuring that all files/assets related to
this test, are neatly included under this test project's directory. This is not
a required step, but a helpful step.

```bash
ln -s ../../../../target/wasm32-unknown-unknown/release/indexer.wasm indexer.wasm
```

### 4. Add a test manifest to the root of the project

Add the following `manifest.yaml` to the root of the `fuel-indexer/` project.

```yaml
namespace: composability_test
graphql_schema: tests/e2e/composable-indexer-test/indexer/schema/schema.graphql
wasm_module: tests/e2e/composable-indexer-test/indexer/indexer.wasm
handlers:
  - event: LogData
    handler: function_one
```

### 5. Start the Fuel client

```bash
cd tests/e2e/composable-indexer-test/

RUST_LOG=info cargo run --bin fuel-node
```

> 5a. If the smart contract address listed by your Fuel node is different from the address listed [in the defaults module](https://github.com/FuelLabs/fuel-indexer/pull/143/files#diff-acab8092cdae5c5f8d849074d1454dd6fb84ed49254698bc885c33a152651bc0R15) you'll need to update `defaults::PING_CONTRACT_ID` accordingly

### 6. Start the Web API

```bash
cd tests/e2e/composable-indexer-test/

RUST_LOG=info cargo run --bin web-api
```

### 7. Start the Postgres service

```bash
docker run -e 'POSTGRES_PASSWORD=postgres' -p 5432:5432 -t postgres:latest
```

### 8. Start the Fuel indexer service

Using the test manifest file created above.

```bash
cd fuel-indexer/

RUST_LOG=info cargo run --bin fuel-indexer \
    -- \
    --fuel-node-host 0.0.0.0 \
    --fuel-node-port 4000 \
    --postgres-host 0.0.0.0 \
    --postgres-password postgres \
    --test-manifest manifest.yaml
```

### 9. Trigger an event


```bash
curl -X POST http://0.0.0.0:8000/ping
```

### 10. Ensure the data was indexed

```bash
echo "SELECT COUNT(*) FROM composability_test.message" | psql "postgres://postgres:postgres@0.0.0.0:5432"
 count
-------
     1
(1 row)
```
