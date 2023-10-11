# greetings-data

This program is used to generate test data for the `greetings` example.

## Usage

```bash
cargo run --bin greetings-data -- --help
```

```text
Test program used to generate data for the greetings example.
greetings-data

USAGE:
    greetings-data [OPTIONS]

OPTIONS:
        --chain-config <CHAIN_CONFIG>    Test wallet filepath
        --contract-bin <CONTRACT_BIN>    Contract bin filepath
    -h, --help                           Print help information
        --host <HOST>                    Host at which to bind.
```

Generate a test data point.

```bash
cargo run --bin greetings-data
```
