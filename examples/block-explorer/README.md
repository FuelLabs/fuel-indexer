# block-explorer

An extremely basic block explorer implementation that shows how blocks, transactions, contracts, and accounts can be persisted into the database.

## Usage

### Spin up containers

Spin up containers for the Postgres indexer service backend, and the indexer service.

> IMPORTANT: Ensure that any local Postgres instance on port 5432 is stopped.

```bash
docker compose up
```

### Deploy the indexer

```bash
forc-index deploy \
   --path explorer-index \
   --target-dir /Users/rashad/development/repos/fuel-indexer \
   --url http://0.0.0.0:29987 \
   --target wasm32-unknown-unknown
```

### Validate

Ensure that test data was indexed via a GraphQL query.

```bash
curl -X POST http://0.0.0.0:29987/api/graph/fuel_examples \
   -H 'content-type: application/json' \
   -d '{"query": "query { tx { id hash status block }}", "params": "b"}' \
| json_pp
```
