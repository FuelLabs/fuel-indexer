# Database

The Fuel Indexer supports the use of [Postgres](https://github.com/docker-library/postgres/blob/2f6878ca854713264ebb27c1ba8530c884bcbca5/14/bullseye/Dockerfile) or SQLite as the database on the data layer.

## Decision Factors

In order to help you choose which option may work best for your use case, here are a number of factors to consider:

- Setup
  - SQLite is a embedded database management solution (DBMS), which allows it to be use it as a serverless database and be included with your application. Postgres uses a client-server model, and requires the database to be set up on a server and a client to be used to interact with the server.
- Supported Data Types
  - SQLite supports [five (5) data types](https://www.sqlite.org/datatype3.html): NULL, BLOB, INTEGER, TEXT, and REAL. Postgres supports a [larger variety of data types](https://www.postgresql.org/docs/current/datatype.html).
- Portability
  - SQLite stores its database as a cross-platform format in a single file on disk, which enables you to copy or move the database to different hosts if necessary. In contrast, if you want to move a Postgres database, you must first export the database to a file and upload it to a separate Postgres database server.
- Authentication
  - SQLite doesn't come with an authentication system, which means that you may have to rely upon the access control capabilities of the underlying operating system in order to protect your database. On the other hand, Postgres allows you to use password-based authentication [and other methods](https://www.postgresql.org/docs/current/auth-methods.html) to ensure secure access.
- Concurrent Access
  - SQLite can be read by an unlimited number of simultaneous readers but only supports one writer at a time. Postgres also supports an unlimited number of simultaneous readers, but unlike SQLite, it supports parallel, non-blocking transactions where possible.

## Indexer Data Representation

- [Types](./types.md)
  - How to use different data types from your Sway contract, all the way to your Postgres table
- [Conventions](./conventions.md)
  - Some of the conventions used in the Fuel Indexer's data layer
- [Foreign Keys](./foreign-keys.md)
  - How foreign keys are handled in GraphQL schema, Postgres, and SQLite
- [Directives](./directives.md)
  - How GraphQL schema directives are translated into data-layer constraints
