# GraphQL

## Schema

[ TODO ]

## API Server

- The `fuel-indexer-api-server` crate of the Fuel indexer contains a standalone GraphQL API server that acts as a queryable endpoint on top of the database.
- Note that the main `fuel-indexer` binary of the indexer project also contains a queryable GraphQL API endpoint.

> The `fuel-indexer-api-server` crate offers a _standalone_ GraphQL API endpoint, whereas the GraphQL endpoint offered in `fuel-indexer` is bundled with other Fuel indexer functionality (e.g., execution, handling, data-layer contruction, etc).

## Usage

To run the standalone Fuel indexer GraphQL API server using a configuration file:

```bash
cargo run --bin fuel-indexer-api-server -- --config config.yaml
```

Where `config.yaml` is based on [the default service configuration file](https://github.com/FuelLabs/fuel-indexer/blob/master/config.yaml).
