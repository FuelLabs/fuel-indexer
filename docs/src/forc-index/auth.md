# `forc index auth`

Authenticate against an indexer operator.

> IMPORTANT: There must be an indexer service running at `--url` in order for this to work.

```bash
forc index auth --account 0
```

```text
USAGE:
    forc-index auth [OPTIONS] --account <ACCOUNT>

OPTIONS:
        --account <ACCOUNT>    Index of account to use for signing.
    -h, --help                 Print help information
        --url <URL>            URL at which to deploy indexer assets. [default:
                               http://localhost:29987]
    -v, --verbose              Verbose output.
```
