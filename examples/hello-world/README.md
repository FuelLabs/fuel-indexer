# hello world

A "Hello World" type of program for the Fuel Indexer service.

## Usage

### Spin up containers

Spin up containers for the Postgres indexer service backend, the indexer service, and a test Fuel node.

> If the `fuel-indexer/local:fuel-node` image is not local and has to be built, this might take a second. If you have the `fuel-indexer/local:fuel-node` image locally, you need not pass the `--build` flag to the command below.
>
> IMPORTANT: Ensure that any local Postgres instance on port 5432 is stopped.

```bash
docker compose up --build
```

### Deploy the indexer

```bash
forc index deploy \
   --path hello-indexer \
   --target-dir $PWD/../../ \
   --url http://0.0.0.0:29987
```

### Interact

Trigger some test data by simulating a contract call.

```bash
cargo run -p hello-world-data --bin hello-world-data -- --host 0.0.0.0:4000
```

### Validate

Ensure that test data was indexed via a GraphQL query:
  1. Open this GraphQL playground link http://localhost:29987/api/playground/fuel_examples/hello_indexer.
  2. Submit the following query

```graphql
query {
   salutation {
      id
      message_hash
      message
      greeter
      first_seen
      last_seen
   }
}
```
