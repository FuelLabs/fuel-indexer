# GraphQL API Server

- The `fuel-indexer-api-server` crate of the Fuel indexer contains a standalone GraphQL API server that acts as a queryable endpoint on top of the database.
- Note that the main `fuel-indexer` binary of the indexer project also contains a queryable GraphQL API endpoint.

> The `fuel-indexer-api-server` crate offers a _standalone_ GraphQL API endpoint, whereas the GraphQL endpoint offered in `fuel-indexer` is bundled with other Fuel indexer functionality (e.g., execution, handling, data-layer contruction, etc).

## Usage

To run the standalone Fuel indexer GraphQL API server using a configuration file:

```bash
fuel-indexer-api-server run --config config.yaml
```

In the above example, `config.yaml` is based on [the default service configuration file](https://github.com/FuelLabs/fuel-indexer/blob/master/config.yaml).

## Options

```text
USAGE:
    fuel-indexer-api-server run [OPTIONS]

OPTIONS:
    -c, --config <CONFIG>
            API server config file.

        --database <DATABASE>
            Database type. [default: postgres] [possible values: postgres]

        --fuel-node-host <FUEL_NODE_HOST>
            Host of the running Fuel node. [default: 127.0.0.1]

        --fuel-node-port <FUEL_NODE_PORT>
            Listening port of the running Fuel node. [default: 4000]

        --graphql-api-host <GRAPHQL_API_HOST>
            GraphQL API host. [default: 127.0.0.1]

        --graphql-api-port <GRAPHQL_API_PORT>
            GraphQL API port. [default: 29987]

    -h, --help
            Print help information

        --log-level <LOG_LEVEL>
            Log level passed to the Fuel Indexer service. [default: info] [possible values: info,
            debug, error, warn]

        --metrics <metrics>
            Use Prometheus metrics reporting. [default: true]

        --postgres-database <POSTGRES_DATABASE>
            Postgres database.

        --postgres-host <POSTGRES_HOST>
            Postgres host.

        --postgres-password <POSTGRES_PASSWORD>
            Postgres password.

        --postgres-port <POSTGRES_PORT>
            Postgres port.

        --postgres-user <POSTGRES_USER>
            Postgres username.

        --run-migrations <run-migrations>
            Run database migrations before starting service. [default: true]

    -V, --version
            Print version information
```
