# Indexer Service Infrastructure

- [Service Components](#components)
- [Fuel Indexer Service](#fuel-indexer-service)
  - [Starting the service via CLI options](#using-cli-options-indexer-service)
  - [Starting the service via a config file](#using-a-configuration-file-indexer-service)
- [Fuel Indexer Web Server](#web-api-server)
  - [Starting the service via CLI options](#using-cli-options-web-server)
  - [Starting the service via a config file](#using-a-configuration-file-web-server)

A Fuel indexer service instance requires just three components:

- a **Fuel Node**: Custom indexers monitor incoming blocks via a Fuel GraphQL server and extract information about the state of the Fuel blockchain.

- a **PostgreSQL database server**: Extracted information is saved into a database.

- a **Web Server**: dApps can query indexers for up-to-date information and operators can deploy/remove indexers as needed.

---

## Components

| Component | Default Host | Default Port | CLI Argument | Environment Variable |
|---|---|---|---|---|
| Fuel Node | localhost | 4000 |    `--fuel-node-{host,port}`    | $FUEL_NODE_{HOST,PORT} |
| Database Server | localhost | 5432 | `--postgres-{username,database,password,host,port}` | $POSTGRES_{USERNAME,DATABASE,PASSWORD,HOST,PORT} |
| Indexer Service Web API | localhost | 29987 | `--web-api-{host,port}` | $WEB_API_{HOST,PORT} |

---

## Fuel Indexer Service

The Fuel indexer service will connect to any Fuel GraphQL server, which means you can run your own node or use a node provided by Fuel. The indexer service web server is included with the Fuel indexer; it's available as soon as the indexer is started through `fuel-indexer run`. The only component that isn't provided for you is a Postgres database server. You should set up a server according to your own needs and specifications.

> You can start the indexer service with an array of CLI options. Note that most (if not all) of these options include sensible defaults.

### Using CLI options (Indexer Service)

```bash
fuel-indexer run --help
```

```text
Standalone binary for the fuel indexer service.

USAGE:
    fuel-indexer run [OPTIONS]

OPTIONS:
        --accept-sql-queries
            Allow the web server to accept raw SQL queries.

        --allow-non-sequential-blocks
            Allow missing blocks or non-sequential block processing.

        --auth-enabled
            Require users to authenticate for some operations.

        --auth-strategy <AUTH_STRATEGY>
            Authentication scheme used.

        --block-page-size <BLOCK_PAGE_SIZE>
            Amount of blocks to return in a request to a Fuel node. [default: 20]

    -c, --config <FILE>
            Indexer service config file.

        --client-request-delay <CLIENT_REQUEST_DELAY>
            Make the service wait for the given duration between block requests to a Fuel client.

        --database <DATABASE>
            Database type. [default: postgres] [possible values: postgres]

        --disable-toolchain-version-check
            By default, Fuel Indexer will only accept WASM indexer modules compiled with the same
            toolchain version as the version of Fuel Indexer.

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
            Max body size for web server requests. [default: 5242880]

        --metering-points <METERING_POINTS>
            The number of WASM opcodes after which the indexer's event handler will stop execution.
            [default: 30000000000]

        --metrics
            Use Prometheus metrics reporting.

        --network <NETWORK>
            Use a network alias when connecting to a Fuel client. [possible values: beta-3, beta-4,
            beta-5]

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

        --remove-data
            When replacing an indexer, also remove the indexed data.

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

### Using a configuration file (Indexer Service)

```yaml
{{#include ../../../config.yaml}}
```

---

## Web API Server

The `fuel-indexer-api-server` crate of the Fuel indexer contains a standalone web server that acts as a queryable endpoint on top of the database. Note that the main `fuel-indexer` binary of the indexer project also contains the same web server endpoint.

> The `fuel-indexer-api-server` crate offers a _standalone_ web server endpoint, whereas the API endpoint offered in `fuel-indexer` is bundled with other Fuel indexer functionality (e.g., execution, handling, data-layer construction, etc). Offering the API server as a separate piece allows users to separate components and run them on different systems, if desired.

### Using CLI Options (Web Server)

> You can start the indexer service with an array of CLI options. Note that most (if not all) of these options include sensible defaults.

```bash
fuel-indexer-api-server run --help
```

```text
Fuel indexer web server

USAGE:
    fuel-indexer-api-server run [OPTIONS]

OPTIONS:
        --accept-sql-queries
            Allow the web server to accept raw SQL queries.

        --auth-enabled
            Require users to authenticate for some operations.

        --auth-strategy <AUTH_STRATEGY>
            Authentication scheme used. [possible values: jwt]

    -c, --config <CONFIG>
            API server config file.

        --database <DATABASE>
            Database type. [default: postgres] [possible values: postgres]

        --disable-toolchain-version-check
            By default, Fuel Indexer will only accept WASM indexer modules compiled with the same
            toolchain version as the version of Fuel Indexer.

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

        --network <NETWORK>
            Use a network alias when connecting to a Fuel client. [possible values: beta-3, beta-4,
            beta-5]

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

### Using A Configuration File (Web Server)

To run the standalone Fuel indexer web server server using a configuration file:

```bash
fuel-indexer-api-server run --config config.yaml
```

In the above example, `config.yaml` is based on [the default service configuration file](https://github.com/FuelLabs/fuel-indexer/blob/develop/config.yaml).
