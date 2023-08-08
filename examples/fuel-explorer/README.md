# fuel-explorer

The Fuel block explorer.

## Usage

> NOTE: Commands are run from `fuel-indexer/examples/fuel-explorer`

### Spin up containers

Spin up containers for the Postgres database server and the indexer service.

> IMPORTANT: Ensure that any local Postgres instance on port 5432 is stopped.

```bash
docker compose up --build
```

### Deploy the indexer

```bash
forc index deploy --path hello-indexer --url http://0.0.0.0:29987
```

### Interact

Since this example does not require a smart contract, we don't need any type of interaction to trigger events.

### Validate

Ensure that test data was indexed via a GraphQL query:
  1. Open this GraphQL playground link http://192.168.1.34:29987/api/playground/fuellabs/explorer
  2. Submit the following query

```graphql
query {
  transactions: transaction(order: { id: desc }, first: 5) {
    id
    time
    label
  }
}
```

> IMPORTANT: Since this example uses a dockerized indexer service, with the GraphQL
> web API being bound at interface `0.0.0.0` your LAN IP might differ from the
> `192.168.1.34` mentioned above.
>
> On *nix platforms you can typically find your LAN IP via `ifconfig | grep inet`