# `forc index start`

Start a local Fuel Indexer service.

```bash
forc index start --background
```

```text
USAGE:
    forc-index start [OPTIONS]

OPTIONS:
        --background               Whether to run the Fuel Indexer in the background.
        --bin <BIN>                Path to the fuel-indexer binary.
        --config <CONFIG>          Path to the config file used to start the Fuel Indexer.
    -h, --help                     Print help information
        --log-level <LOG_LEVEL>    Log level passed to the Fuel Indexer service. [default: info]
                                   [possible values: info, debug, error, warn]
```
