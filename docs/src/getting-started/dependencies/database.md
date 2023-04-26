# Database

The Fuel indexer requires the use of a database. We currently support [PostgresSQL](https://www.postgresql.org/docs/).

## PostgreSQL

> IMPORTANT: Fuel Indexer users on most platforms don't need to explicitly install PostgresQL software via a package manager. When starting the indexer service via `forc index start` simply pass the `--embedded-database` flag in order to have the indexer service download and start an embedded PostgresQL instance via [`forc index postgres`](./../../reference-guide/plugins/forc-postgres/index.md).
>
> However if users or devs would like to install PostgresQL via some package manager, feel free to checkout the more detailed installation steps below.

### macOS

On macOS systems, you can install PostgreSQL through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). 

Once installed, you can add PostgreSQL to your system by running `brew install postgresql`. 
