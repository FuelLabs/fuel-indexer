# hello-world-data

This program is used to generate test data for the `hello-world` example.

## Usage

```bash
cargo run --bin hello-world-data -- --help
```

```text
Test program used to generate data for the hello-world example.
hello-world-data

USAGE:
    hello-world-data [OPTIONS]

OPTIONS:
        --chain-config <CHAIN_CONFIG>    Test wallet filepath
        --contract-bin <CONTRACT_BIN>    Contract bin filepath
    -h, --help                           Print help information
        --host <HOST>                    Host at which to bind.
```

Generate a test data point.

```bash
cargo run --bin hello-world-data
```
