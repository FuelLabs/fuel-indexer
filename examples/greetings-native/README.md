# greetings-native

A simple program that demonstrates the full Fuel indexer experience.

## Usage

> NOTE: Commands are run from `fuel-indexer/examples/greetings-native`

### Spin up containers

Build image locally

```bash
docker compose up
```

Spin up containers for the Postgres database server and the indexer service.

> IMPORTANT: Ensure that any local Postgres instance on port 5432 is stopped.

```bash
docker compose up
```

### Deploy the indexer

> Note that since this example uses native execution (as opposed to WASM execution), there is no need to
> "deploy" the indexer. You'll notice that your indexer is already running inside your Docker container.

### Interact

Trigger some test data by simulating a contract call.

```bash
cargo run -p greetings-data --bin greetings-data -- --host 0.0.0.0:4000
```

### Validate

Ensure that test data was indexed via a GraphQL query:
  1. Open this GraphQL playground link http://192.168.1.34:29987/api/playground/fuellabs/greetings-native_indexer
  2. Submit the following query

```graphql
query {
   transaction {
      hash
      block {
        hash
        height
      }
   }
}
```

> IMPORTANT: Since this example uses a dockerized indexer service, with the GraphQL
> web server being bound at interface `0.0.0.0` your LAN IP might differ from the
> `192.168.1.34` mentioned above.
>
> On *nix platforms you can typically find your LAN IP via `ifconfig | grep inet`
