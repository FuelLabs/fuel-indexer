# `forc index kill`

Kill the indexer process. Note that this command will kill any process listening on the default indexer port or the port specified by the `--port` flag.

```bash
forc index kill --port 29987
```

```text
USAGE:
    forc-index kill [OPTIONS]

OPTIONS:
    -9                   Terminate or kill
    -h, --help           Print help information
        --port <PORT>    Port at which to detect indexer service API is running. [default: 29987]
```
