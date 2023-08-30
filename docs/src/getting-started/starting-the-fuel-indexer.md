# Indexer Service Infrastructure

A Fuel indexer service instance requires just three components:

- a **Fuel node**: Custom indexers monitor incoming blocks from a Fuel node and extract information about the state of the Fuel blockchain.
- a **Postgres database server**: Extracted information is saved into a database.
- the **indexer service web API**: dApps can query indexers for up-to-date information and operators can deploy/remove indexers as needed.

The Fuel indexer service will connect to any Fuel node, which means you can run your own node or use a node provided by Fuel. The indexer service web API is included with the Fuel indexer; it's available as soon as the indexer is started through `fuel-indexer run`. The only component that isn't provided for you is a Postgres database server. You should set up a server according to your own needs and specifications.

## Components

| Component | Default Host | Default Port | CLI Argument | Environment Variable |
|---|---|---|---|---|
| Fuel Node | localhost | 4000 | `--fuel-node-host` / `--fuel-node-port` |  |
| Database Server | localhost | 5432 | `--postgres-user` / `--postgres--password` / `--postgres-host` / `--postgres--port` / `--postgres-database` |  |
| Indexer Service Web API | localhost | 29987 | `--web-api-host` / `--web-api-port` |  |

## Starting the Fuel indexer
### Using CLI options

```text
Standalone binary for the fuel indexer service.

USAGE:
    fuel-indexer run [OPTIONS]

OPTIONS:
        --accept-sql-queries
            Allow the web API to accept raw SQL queries.

        --auth-enabled
            Require users to authenticate for some operations.

        --auth-strategy <AUTH_STRATEGY>
            Authentication scheme used.

        --block-page-size <BLOCK_PAGE_SIZE>
            Amount of blocks to return in a request to a Fuel node. [default: 10]

    -c, --config <FILE>
            Indexer service config file.

        --database <DATABASE>
            Database type. [default: postgres] [possible values: postgres]

        --embedded-database
            Automatically create and start database using provided options or defaults.

        --fuel-node-host <FUEL_NODE_HOST>
            Host of the running Fuel node. [default: localhost]

        --fuel-node-port <FUEL_NODE_PORT>
            Listening port of the running Fuel node. [default: 4000]

    -h, --help
            Print help information

        --indexer-net-config
            Allow network configuration via indexer manifests.

        --jwt-expiry <JWT_EXPIRY>
            Amount of time (seconds) before expiring token (if JWT scheme is specified).

        --jwt-issuer <JWT_ISSUER>
            Issuer of JWT claims (if JWT scheme is specified).

        --jwt-secret <JWT_SECRET>
            Secret used for JWT scheme (if JWT scheme is specified).

        --local-fuel-node
            Start a local Fuel node.

        --log-level <LOG_LEVEL>
            Log level passed to the Fuel Indexer service. [default: info] [possible values: info,
            debug, error, warn]

    -m, --manifest <FILE>
            Indexer config file.

        --max-body-size <MAX_BODY_SIZE>
            Max body size for web API requests. [default: 5242880]

        --metering-points <METERING_POINTS>
            The number of WASM opcodes after which the indexer's event handler will stop execution.
            [default: 30000000000]

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

        --replace-indexer
            Whether to allow replacing an existing indexer. If not specified, an attempt to deploy
            over an existing indexer results in an error.

        --run-migrations
            Run database migrations before starting service.

        --stop-idle-indexers
            Prevent indexers from running without handling any blocks.

    -v, --verbose
            Enable verbose logging.

    -V, --version
            Print version information

        --web-api-host <WEB_API_HOST>
            Web API host. [default: localhost]

        --web-api-port <WEB_API_PORT>
            Web API port. [default: 29987]

```

### Using a configuration file

```yaml
{{#include ../../../config.yaml}}
```
