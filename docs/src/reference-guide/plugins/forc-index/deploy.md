# `forc index deploy`

Deploy a given indexer project to a particular endpoint

```bash
forc index deploy --url https://index.swaysway.io --manifest my_index.manifest.yaml
```

```text
USAGE:
    forc-index deploy [OPTIONS] --manifest <MANIFEST>

OPTIONS:
        --auth <AUTH>            Authentication header value.
    -h, --help                   Print help information
        --manifest <MANIFEST>    Path of the indexer manifest to upload.
        --url <URL>              URL at which to upload indexer assets. [default:
                                 http://127.0.0.1:29987]
```
