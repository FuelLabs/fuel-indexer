# `forc index revert`

Revert the running indexer to the previous version.

```bash
forc index revert --manifest my_indexer.manifest.yaml
```

```text
USAGE:
    forc-index revert [OPTIONS]

OPTIONS:
        --auth <AUTH>            Authentication header value.
    -h, --help                   Print help information
    -m, --manifest <MANIFEST>    Path to the manifest of indexer project being deployed.
    -p, --path <PATH>            Path of indexer project.
        --url <URL>              URL at which indexer is deployed. [default: http://127.0.0.1:29987]
```
