# GraphQL Server

The GraphQL API server component of the Fuel Indexer is a separate standalone service that acts as a queryable endpoint on top of the data layer.

```bash
cd fuel-indexer/

cargo run --bin api_server -- --config ./config.yaml
```

Where `config.yaml` is based on [the example configuration file](#).
