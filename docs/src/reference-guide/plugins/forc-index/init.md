# `forc index init`

Create a new indexer project at the provided path. If no path is provided the current working directory will be used.

```bash
forc index init --namespace fuel
```

```text
USAGE:
    forc-index init [OPTIONS] --namespace <NAMESPACE>

OPTIONS:
        --absolute-paths           Resolve indexer asset filepaths using absolute paths.
    -h, --help                     Print help information
        --name <NAME>              Name of indexer.
        --namespace <NAMESPACE>    Namespace in which indexer belongs.
        --native                   Initialize an indexer with native execution enabled.
    -p, --path <PATH>              Path at which to create indexer.
```
