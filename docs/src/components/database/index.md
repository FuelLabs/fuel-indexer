# Database

The Fuel indexer uses [Postgres](https://github.com/docker-library/postgres/blob/2f6878ca854713264ebb27c1ba8530c884bcbca5/14/bullseye/Dockerfile) as the primary database.

> Note: The Fuel indexer also offers SQLite support. However, we recommend Postgres usage for any type of production service.

- [Types](./types.md)
  - How to use different data types from your Sway contract, all the way to your Postgres table
- [Conventions](./conventions.md)
  - Some of the conventions used in the Fuel Indexer's data layer
- [Foreign Keys](./foreign-keys.md)
  - How foreign keys are handled in GraphQL schema, Postgres, and SQLite
- [Directives](./directives.md)
  - How GraphQL schema directives are translated into data-layer constraints
