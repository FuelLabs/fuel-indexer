# hello world

A "Hello World" type of program for the Fuel Indexer service.

## Usage

> NOTE: Commands are run from `fuel-indexer/examples/hello-world`

### Spin up containers

Spin up containers for the Postgres database server and the indexer service.

> NOTE: Ensure that any local Postgres instance on port 5432 is stopped.
>
> IMPORTANT: If you're on a Mac M1/M2 please update the `image` for the `fuel-indexer` service in your
> `docker-compose.yaml` to [`ra0x3/fuel-indexer:latest`](https://hub.docker.com/repository/docker/ra0x3/fuel-indexer/general) until [#1099](https://github.com/FuelLabs/fuel-indexer/issues/1099) is resolved.

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
  1. Open this GraphQL playground link http://192.168.1.34:29987/api/playground/fuel_examples/hello_indexer
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

> IMPORTANT: Since this example uses a dockerized indexer service, with the GraphQL
> web API being bound at interface `0.0.0.0` your LAN IP might differ from the
> `192.168.1.34` mentioned above.
>
> On *nix platforms you can typically find your LAN IP via `ifconfig | grep inet`
