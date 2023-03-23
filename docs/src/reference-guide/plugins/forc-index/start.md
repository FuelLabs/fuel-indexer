# `forc index start`

Start a local Fuel Indexer service.

```bash
forc index start
```

```text
USAGE:
    forc-index start [OPTIONS]

OPTIONS:
        --auth-enabled
            Require users to authenticate for some operations.

        --auth-strategy <AUTH_STRATEGY>
            Authentication scheme used.

    -c, --config <FILE>
            Indexer service config file.

        --database <DATABASE>
            Database type. [default: postgres] [possible values: postgres]

        --embedded-database
            Automatically create and start database using provided options or defaults.

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

        --jwt-expiry <JWT_EXPIRY>
            Amount of time (seconds) before expiring token (if JWT scheme is specified).

        --jwt-issuer <JWT_ISSUER>
            Issuer of JWT claims (if JWT scheme is specified).

        --jwt-secret <JWT_SECRET>
            Secret used for JWT scheme (if JWT scheme is specified).

        --log-level <LOG_LEVEL>
            Log level passed to the Fuel Indexer service. [default: info] [possible values: info,
            debug, error, warn]

    -m, --manifest <FILE>
            Index config file.

        --max-body-size <MAX_BODY_SIZE>
            Max body size for GraphQL API requests. [default: 5242880]

        --metrics
            Use Prometheus metrics reporting.

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

        --run-migrations
            Run database migrations before starting service.

        --stop-idle-indexers
            Prevent indexers from running without handling any blocks.

    -V, --version
            Print version information

        --verbose
            Enable verbose logging.
```
