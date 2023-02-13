# Starting the Fuel Indexer

## Using CLI options

```text
USAGE:
    fuel-indexer run [OPTIONS]

OPTIONS:
    -c, --config <CONFIG>
            Indexer service config file.

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
            Log level passed to the Fuel Indexer service. [default: info]
                [possible values: info, debug, error, warn]

    -m, --manifest <MANIFEST>
            Index config file.

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

## Using a configuration file

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

## Database configuration options.

database:
    postgres:
        user: postgres
        database:
        password:
        host: 127.0.0.1
        port: 5432

metrics: true
```
