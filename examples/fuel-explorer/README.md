# fuel-explorer

The Fuel block explorer.

## Usage

> NOTE: Commands are run from `fuel-indexer/examples/fuel-explorer`

### Spin up containers

Spin up containers for the Postgres database server and the indexer service.

> IMPORTANT: If you're on a platform using Apple Silicon, you will have to build your image locally, rather than pulling an image from the image registry. We have [an open issue](https://github.com/FuelLabs/fuel-indexer/issues/578) to add support for ARM Arch64 images.
>
> In order to use a local image you can just:
>   1. Build your image locally: `docker build -t fuel-indexer/local:latest -f deployment/Dockerfile .`
>   2. Update your `docker-compose.yaml` file with `image: fuel-indexer/local:latest` for the `fuel-indexer` service.
>
> IMPORTANT: Ensure that any local Postgres instance on port 5432 is stopped.

```bash
docker compose up --build
```

### Deploy the indexer

```bash
forc index deploy \
   --path fuel-explorer \
   --target-dir $PWD/../../ \
   --url http://0.0.0.0:29987
```

### Interact

Since this example does not require a smart contract, we don't need any type of interaction to trigger events.

### Validate

Ensure that test data was indexed via a GraphQL query:
  1. Open this GraphQL playground link http://192.168.1.34:29987/api/playground/fuel/explorer
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