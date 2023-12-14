# `forc index auth`

Authenticate against an indexer operator.

> IMPORTANT: There must be an indexer service running at `--url` in order for this to work.

```bash
forc index auth --account 0
```

```text
Authenticate against an indexer service

USAGE:
    forc-index auth [OPTIONS]

OPTIONS:
        --account <ACCOUNT>    Index of account to use for signing. [default: 0]
    -h, --help                 Print help information
        --url <URL>            URL at which to deploy indexer assets. [default:
                               http://127.0.0.1:29987]
    -v, --verbose              Verbose output.
```
