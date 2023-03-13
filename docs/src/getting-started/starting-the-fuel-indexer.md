# Starting the Fuel Indexer

## Using CLI options

```text
Standalone binary for the fuel indexer service.

USAGE:
    fuel-indexer run [OPTIONS]

OPTIONS:
        --auth-enabled <auth-enabled>
            Require users to authenticate for some operations. [default: false]

        --auth-strategy <AUTH_STRATEGY>
            Authentication scheme used.

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

## Using a configuration file

```yaml
# # The following is an example Fuel indexer configuration file.
# #
# # This configuration spec is intended to be used for a single instance
# # of a Fuel indexer node or service.
# #
# # For more info on how the Fuel indexer works, read the book: https://fuellabs.github.io/fuel-indexer/master/
# # or specifically read up on these configuration options: https://fuellabs.github.io/fuel-indexer/master/getting-started/configuration.html

# # Use Prometheus metrics reporting.
# metrics: true

# # Prevent indexers from running without handling any blocks.
# stop_idle_indexers: true

# # ***********************
# # Fuel Node configuration
# # ************************

# fuel_node:

#   # Host of the running Fuel node.
#   host: 127.0.0.1

#   # Listening port of the running Fuel node.
#   port: 4000

# # *************************
# # GraphQL API configuration
# # *************************

# graphql_api:
#   # GraphQL API host.
#   host: 127.0.0.1

#   # GraphQL API port.
#   port: 29987

#   # Run database migrations before starting service.
#   run_migrations: false

#   # Max body size for GraphQL API requests.
#   max_body_size: "5242880"

# # *******************************
# # Database configuration options.
# # *******************************

# database:

#   postgres:
#     # Postgres username.
#     user: postgres

#     # Postgres database.
#     database: postgres

#     # Postgres password.
#     password: password

#     # Postgres host.
#     host: 127.0.0.1

#     # Postgres port.
#     port: 5432

# # ******************************
# # Indexer service authentication
# # ******************************

# authentication:
#   # Require users to authenticate for some operations.
#   enabled: false

#   # Which authentication scheme to use.
#   strategy: JWT

#   # Secret used if JWT authentication is specified.
#   jwt_secret: abcdefghijklmnopqrstuvwxyz1234567890*

#   # JWT issuer if JWT authentication is specified.
#   # jwt_issuer: FuelLabs

#   # Amount of time (seconds) before expiring token if JWT authentication is specified.
#   # jwt_expiry: 2592000
```
