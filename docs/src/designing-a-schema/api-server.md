# GraphQL API Server

- The `fuel-indexer-api-server` crate of the Fuel indexer contains a standalone GraphQL API server that acts as a queryable endpoint on top of the database.
- Note that the main `fuel-indexer` binary of the indexer project also contains a queryable GraphQL API endpoint.

> The `fuel-indexer-api-server` crate offers a _standalone_ GraphQL API endpoint, whereas the GraphQL endpoint offered in `fuel-indexer` is bundled with other Fuel indexer functionality (e.g., execution, handling, data-layer contruction, etc).

## Usage

To run the standalone Fuel indexer GraphQL API server using a configuration file:

```bash
fuel-indexer-api-server run --config config.yaml
```

In the above example, `config.yaml` is based on [the default service configuration file](https://github.com/FuelLabs/fuel-indexer/blob/develop/config.yaml).

## Options

```text
Fuel indexer web API

USAGE:
    fuel-indexer-api-server run [OPTIONS]

OPTIONS:
        --accept-sql-queries
            Allow the web API to accept raw SQL queries.

        --auth-enabled
            Require users to authenticate for some operations.

        --auth-strategy <AUTH_STRATEGY>
            Authentication scheme used. [possible values: jwt]

    -c, --config <CONFIG>
            API server config file.

        --database <DATABASE>
            Database type. [default: postgres] [possible values: postgres]

        --fuel-node-host <FUEL_NODE_HOST>
            Host of the running Fuel node. [default: localhost]

        --fuel-node-port <FUEL_NODE_PORT>
            Listening port of the running Fuel node. [default: 4000]

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

        --max-body-size <MAX_BODY_SIZE>
            Max body size for web requests. [default: 5242880]

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

        --rate-limit
            Enable rate limiting.

        --rate-limit-request-count <RATE_LIMIT_REQUEST_COUNT>
            Maximum number of requests to allow over --rate-limit-window..

        --rate-limit-window-size <RATE_LIMIT_WINDOW_SIZE>
            Number of seconds over which to allow --rate-limit-rps.

        --run-migrations
            Run database migrations before starting service.

    -v, --verbose
            Enable verbose logging.

    -V, --version
            Print version information

        --web-api-host <WEB_API_HOST>
            Web API host. [default: localhost]

        --web-api-port <WEB_API_PORT>
            Web API port. [default: 29987]
```
