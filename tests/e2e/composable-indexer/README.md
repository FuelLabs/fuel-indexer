# Composable Indexer

An end-to-end test of the indexer service using a reproducible environment.

### 1. Spin up services
> You'll need to stop your local Postgres on `5432` if you have one running.

```bash
bash tests/e2e/composable-indexer/compose-up.bash
```

### 2. Trigger an event

```bash
curl -X POST http://0.0.0.0:8000/ping
```

### 3. Confirm event was indexed

```bash
curl -X POST http://0.0.0.0:29987/graph/composability_test \
   -H 'content-type: application/json' \
   -d '{"query": "query { message { id ping pong message }}", "params": "b"}' \
   | json_pp
```

Should output

```json
[
   {
      "id" : 1,
      "message" : "6161616173646673646661736466736466616173646673646661736466736466",
      "ping" : 123,
      "pong" : 456
   }
]
```
