# Configuration

When using [`forc index`](../forc-index/index.md), there are a variety of command-line flags that you can use to configure the Fuel indexer service and API server.

```yaml
## The following is an example Fuel indexer configuration file.
##
## This configuration spec is intended to be used for a single instance
## of a Fuel indexer node or service.

## Fuel Node configuration

fuel_node:
    host: 127.0.0.1
    port: 4000

## GraphQL API configuration

graphql_api:
    host: 127.0.0.1
    port: 29987
    run_migrations: false

## Database configuration options. Use either the Postgres
## configuration or the SQLite configuration, but not both

database:
    postgres:
        user: postgres
        database:
        password:
        host: 127.0.0.1
        port: 5432

    sqlite:
        path: database/sqlite/sqlite.db

metrics: true
```
