# Indexer Configuration

- Below you will find a list of CLI configuration options that can be used to configure either the Fuel indexer service, the standalone Fuel indexer GraphQL API service, or both. 
- For those who prefer using a configuration file, you can checkout the [default service configuration file](https://github.com/FuelLabs/fuel-indexer/blob/master/config.yaml), which also shows the default values used for these configuration options.

## Usage

### Using the `fuel-indexer` binary

`./fuel-indexer [options]`

### Using the `fuel-indexer-api-server` binary

`./fuel-indexer-api-server [options]`

### Options

`-c` `--config`

- Path to the configuration file.

`-m` `--manifest`

- Path to manifest file from which initial indices will be loaded

> Fuel node: The node running the Fuel client implementation.

`--fuel-node-host` <FUEL-NODE-HOST>

- IP of the Fuel node

`--fuel-node-port` <FUEL-NODE-PORT>

- Port of the Fuel node

> GraphQL API: The enpoint at which GraphQL queries will be processed. This is context dependent. If ran
using the `fuel-indexer` binary, these options apply to the GraphQL service run in that binary. If ran using
the `fuel-indexer-api-server` binary, these options will apply to that service.

`--graphql-api-host` <GRAPHQL-API-HOST>

- IP at which to bind the GraphQL server

`--graphql-api-port` <GRAPHQL-API-PORT>

- Port at which to bind the GraphQL server

- `--run-migrations` <RUN-MIGRATIONS>

- Whether to run the migrations on the GraphQL API's connected database

> Postgres: Standard Postgres connection options.

`--postgres-host` <POSTGRES-HOST>

- Postgres host

`--postgres-port` <POSTGRES-PORT>

- Postgres port

`--postgres-username` <POSTGRES-USERNAME>

- Postgres username

`--postgres-password` <POSTGRES-PASSWORD>

- Postgres password (redacted from logging)

`--postgres-database` <POSTGRES-DATABASE>

- Postgres database

> SQLite: An alternative database implementation using standard SQLite connection options

`--sqlite-database` <SQLITE-DATABASE>

- Path to SQLite database