# Database

The Fuel indexer uses [PostgreSQL](https://github.com/docker-library/postgres/blob/2f6878ca854713264ebb27c1ba8530c884bcbca5/14/bullseye/Dockerfile) as the primary database. We're open to supporting other storage solutions in the future.

In this chapter, you can find information regarding how your data should be structured for use in the Fuel indexer:

- [Foreign Keys](./foreign-keys.md)
  - How foreign keys are handled in the Fuel indexer.
- [⚠️ IDs](./ids.md)
  - Explains some conventions surrounding the usage of `ID` types
