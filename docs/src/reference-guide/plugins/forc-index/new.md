# `forc index new`

Create a new indexer project in a new directory.

```bash
forc index new --namespace fuel --path /home/fuel/projects
```

```text
USAGE:
    forc-index new [OPTIONS] --namespace <NAMESPACE> <PATH>

ARGS:
    <PATH>    Path at which to create indexer

OPTIONS:
        --absolute-paths           Resolve indexer asset filepaths using absolute paths.
    -h, --help                     Print help information
        --name <NAME>              Name of indexer.
        --namespace <NAMESPACE>    Namespace to which indexer belongs.
        --native                   Whether to initialize an indexer with native execution enabled.
    -v, --verbose <verbose>        Enable verbose output. [default: true]

```
