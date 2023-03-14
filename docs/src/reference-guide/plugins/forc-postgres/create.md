# `forc index postgres create`

Create a new database.

```text
forc index postgres create example_database
```

```text
USAGE:
    forc-index postgres create [OPTIONS] <NAME>

ARGS:
    <NAME>    Name of database.

OPTIONS:
        --auth-method <AUTH_METHOD>
            Authentication method. [default: plain] [possible values: plain, md5, scram-sha-256]

    -c, --config <CONFIG>
            Fuel indexer configuration file.

        --database-dir <DATABASE_DIR>
            Where to store the PostgreSQL database.

    -h, --help
            Print help information

        --migration-dir <MIGRATION_DIR>
            The directory containing migration scripts.

    -p, --password <PASSWORD>
            Database password. [default: postgres]

    -p, --port <PORT>
            Port to use. [default: 5432]

        --persistent
            Do not clean up files and directories on database drop.

        --postgres-version <POSTGRES_VERSION>
            PostgreSQL version to use. [default: v14] [possible values: v15, v14, v13, v12, v11,
            v10, v9]

        --start
            Start the PostgreSQL instance after creation.

        --timeout <TIMEOUT>
            Duration to wait before terminating process execution for pg_ctl start/stop and initdb.

    -u, --user <USER>
            Database user. [default: postgres]
```
