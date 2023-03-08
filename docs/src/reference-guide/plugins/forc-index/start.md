# `forc index start`

Start a local Fuel Indexer service.

```bash
forc index start
```

```text
USAGE:
    forc-index start [OPTIONS]

OPTIONS:
        --auto-setup-database
            Automatically create and start database using provided options or defaults.

    -c, --config <FILE>
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
            Log level passed to the Fuel Indexer service. [default: info] [possible values: info,
            debug, error, warn]

    -m, --manifest <FILE>
            Index config file.

        --max-body <MAX_BODY>
            Max body size for the GraphQL API [default: 5242880]

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

        --stop-idle-indexers
            Prevent indexers from running without handling any blocks.

    -V, --version
            Print version information
```
