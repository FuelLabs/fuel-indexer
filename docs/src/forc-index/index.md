# forc index

`forc index` is the recommended method for end users to interact with the Fuel indexer. After you have installed `fuelup`, you can run the `forc index help` command in your terminal to view the available commands.

```text
forc index help
```

```text
USAGE:
    forc-index <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    auth        Authenticate against an indexer service
    build       Build an indexer
    check       Check for Fuel indexer components
    deploy      Deploy an indexer to an indexer service
    help        Print this message or the help of the given subcommand(s)
    kill        Kill the indexer process. Note that this command will kill any process listening
                    on the default indexer port or the port specified by the `--port` flag
    new         Create a new indexer project in a new directory
    postgres    Fuel Postgres Orchestrator
    remove      Stop and remove a running indexer
    start       Standalone binary for the Fuel indexer service
    status      Check the status of a registered indexer
```
