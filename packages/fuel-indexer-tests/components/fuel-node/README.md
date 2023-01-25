# fuel-node

An ephemeral Fuel node used for testing.

## Usage

```bash
cargo run --bin fuel-node -- --help
```

```text
An ephemeral Fuel node used for testing.
fuel-node

USAGE:
    fuel-node [OPTIONS]

OPTIONS:
        --chain-config <CHAIN_CONFIG>    Test wallet filepath
        --contract-bin <CONTRACT_BIN>    Contract bin filepath
    -h, --help                           Print help information
        --host <HOST>                    Host at which to bind.
```

Start the node

```bash
cargo run --bin fuel-node
```
