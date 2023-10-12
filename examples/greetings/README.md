# greetings

A simple program that demonstrates the full Fuel indexer experience.

## Usage

> NOTE: Commands are run from `fuel-indexer/examples/greetings`

### Spin up containers

Pull the latest image 

```bash
docker pull ghcr.io/fuellabs/fuel-indexer:latest
```

Spin up containers for the Postgres database server and the indexer service.

> IMPORTANT: Ensure that any local Postgres instance on port 5432 is stopped.

```bash
docker compose up
```

### Deploy the indexer

```bash
forc index deploy --path greetings --url http://0.0.0.0:29987
```

### Interact

Trigger some test data by simulating a contract call.

```bash
cargo run -p greetings-data --bin greetings-data -- --host 0.0.0.0:4000
```

### Validate

Ensure that test data was indexed via a GraphQL query:
  1. Open this GraphQL playground link http://192.168.1.34:29987/api/playground/fuellabs/greetings_indexer
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
> web server being bound at interface `0.0.0.0` your LAN IP might differ from the
> `192.168.1.34` mentioned above.
>
> On *nix platforms you can typically find your LAN IP via `ifconfig | grep inet`
