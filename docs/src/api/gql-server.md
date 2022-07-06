# GraphQL Server

The `api_server` crate of the Fuel Indexer contains a standalone GraphQL API server that acts as a queryable endpoint on top of the data layer.

Note that the main binary of the `fuel_indexer` crate _also_ contains a queryable GraphQL API endpoint. However, the `api_server` crate offers a standalone GraphQL API endpoint, whereas the `fuel_indexer` bundles its GraphQL API endpoint with other Fuel Indexer functionality (e.g., execution, handling, data-layer contruction, etc).

To run the standalone Fuel Indexer GraphQL API server:

```bash
cd fuel-indexer/

RUST_LOG=debug cargo run --bin api_server -- --config api_server/config.yaml
```

Where `config.yaml` is based on [the default service configuration file](https://github.com/FuelLabs/fuel-indexer/blob/master/api_server/config.yaml).
