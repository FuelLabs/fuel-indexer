# `forc index postgres drop`

Drop a database.

```text
forc index postgres drop example_database
```

```text
USAGE:
    forc-index postgres drop [OPTIONS] <NAME>

ARGS:
    <NAME>    Name of database.

OPTIONS:
    -c, --config <CONFIG>
            Fuel indexer configuration file.

        --database-dir <DATABASE_DIR>
            Where the PostgreSQL database is stored.

    -h, --help
            Print help information

        --remove-persisted
            Remove all database files that might have been persisted to disk.
```
