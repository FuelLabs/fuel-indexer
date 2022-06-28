## Application dependencies

We'll need to install the [`diesel` CLI](https://github.com/diesel-rs/diesel/tree/HEAD/diesel_cli)

```bash
cargo install diesel_cli --no-default-features --features "postgres sqlite"
```

And we'll run the migrations to create our graph registry, types, and columns. Note that this part assumes that you're familiar with basic steps involved in [getting a postgres user/role and database setup.](https://medium.com/coding-blocks/creating-user-database-and-adding-access-on-postgresql-8bfcd2f4a91e)

```bash
createdb -U postgres indexer

cd fuel-indexer/

DATABASE_URL="postgres://postgres@127.0.0.1:5432/indexer" \
    diesel migration run --migration-dir=./schema/migrations
```
