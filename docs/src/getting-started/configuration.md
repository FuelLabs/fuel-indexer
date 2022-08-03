# Indexer Configuration

Below you will find a list of CLI configuration options that can be used to configure either the Fuel Indexer service, the standalone Fuel Indexer GraphQL API service, or both. For those who prefer using a configuration file,
you can [checkout the default service configuration file](https://github.com/FuelLabs/fuel-indexer/blob/master/config.yaml), which also shows the default values used for these configuration options.

## Usage:

Using the main Fuel Indexer service binary.

`cargo run --bin fuel-indexer -- [options]`

Using the standalone GraphQL API server.

`cargo run --bin fuel-indexer-api-server -- [options]`


### Options:

`-c` `--config`

- Path to the configuration file.

`-t` `--test-manifest`

- Path to the test manifest file.

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
