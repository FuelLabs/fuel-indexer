# Database

At this time, the Fuel indexer requires the use of a database. We currently support a single database option: PostgresSQL. PostgreSQL is a database solution with a complex feature set and requires a database server.

## PostgreSQL

> Note: The following explanation is for demonstration purposes only. For an even faster setup on some platforms, you can use the [forc index postgres](../../reference-guide/plugins/forc-postgres/index.md) command. _A production setup should use secure users, permissions, and passwords._

### macOS

On macOS systems, you can install PostgreSQL through Homebrew. If it isn't present on your system, you can install it according to the [instructions](https://brew.sh/). Once installed, you can add PostgreSQL to your system by running `brew install postgresql`. You can then start the service through `brew services start postgresql`. You'll need to create a database for your index data, which you can do by running `createdb [DATABASE_NAME]`. You may also need to create the `postgres` role; you can do so by running `createuser -s postgres`.

### Linux

For Linux-based systems, the installation process is similar. First, you should install PostgreSQL according to your distribution's instructions. Once installed, there should be a new `postgres` user account; you can switch to that account by running `sudo -i -u postgres`. After you have switched accounts, you may need to create a `postgres` database role by running `createuser --interactive`. You will be asked a few questions; the name of the role should be `postgres` and you should elect for the new role to be a superuser. Finally, you can create a database by running `createdb [DATABASE_NAME]`.

In either case, your PostgreSQL database should now be accessible at `postgres://postgres@127.0.0.1:5432/[DATABASE_NAME]`.
