# `forc index pull-abi`

Pull an abi file from a raw GitHub URL.

```bash
forc index pull-abi --raw-url https://raw.githubusercontent.com/rvmelkonian/sway-escrow/main/out/debug/escrow-contract-abi.json
```

```text
USAGE:
    forc-index pull-abi [OPTIONS] --raw-url <RAW_URL>

OPTIONS:
        --contract-name <CONTRACT_NAME>    Name of contract.
    -h, --help                             Print help information
    -p, --path <PATH>                      Path at which to write the ABI.
        --url <URL>                        URL of the ABI file.
    -v, --verbose                          Enable verbose output.
        --with-abi <WITH_ABI>              Only pull the ABI for the given contract.
        --with-contract <WITH_CONTRACT>    Pull the full contract code including the abi.
```
