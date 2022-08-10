# Install the sqlx-cli tool

```cargo install sqlx-cli```

## Quick set up with postgres

```
docker run --rm -p 5432:5432 --name postgres-db -e POSTGRES_PASSWORD=my-secret -d postgres
```

## Create a database
```console

DATABASE_URL=postgres://postgres:my-secret@localhost/mydb sqlx database create
DATABASE_URL=sqlite://mydb.db sqlx database create

```

## Upgrading the schema

-r will make a reversible migration with up.sql and down.sql
```console

sqlx migrate add -r <migration_name>

```

edit migration schema, which will be in `migrations/<YYYYMMDDHHmmSS>_<migration_name>.up.sql` and `down.sql`
