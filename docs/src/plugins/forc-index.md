# forc-index

A `forc` plugin for basic Fuel Indexer interaction.

## Commands

### `forc index init`

Create a new index project at the provided path. If no path is provided the current working directory will be used.

```bash
forc index init --namespace fuel
```

```text
USAGE:
    forc-index init [OPTIONS]

OPTIONS:
    -h, --help                     Print help information
        --name <NAME>              Name of index.
        --namespace <NAMESPACE>    Namespace in which index belongs.
        --native                   Whether to initialize an index with native execution enabled.
    -p, --path <PATH>              Path at which to create index.
```

### `forc index new`

Create new index project at the provided path.

```bash
forc index new --namespace fuel --path /home/fuel/projects
```

```text
USAGE:
    forc-index new [OPTIONS] <PATH>

ARGS:
    <PATH>    Path at which to create index

OPTIONS:
    -h, --help                     Print help information
        --name <NAME>              Name of index.
        --namespace <NAMESPACE>    Namespace in which index belongs.
        --native                   Whether to initialize an index with native execution enabled.
```

### `forc index start`

Start a local Fuel Indexer service.

```bash
forc index start --background
```

```text
USAGE:
    forc-index start [OPTIONS]

OPTIONS:
        --background               Whether to run the Fuel Indexer in the background.
        --bin <BIN>                Path to the fuel-indexer binary.
        --config <CONFIG>          Path to the config file used to start the Fuel Indexer.
    -h, --help                     Print help information
        --log-level <LOG_LEVEL>    Log level passed to the Fuel Indexer service. [default: info]
                                   [possible values: info, debug, error, warn]
```

### `forc index deploy`

Deploy a given index project to a particular endpoint

```bash
forc index deploy --url https://index.swaysway.io --manifest my_index.manifest.yaml
```

### `forc index remove`

Stop and remove a running index

```bash
forc index remove --url https://index.swayswap.io --manifest my_index.manifest.yaml
```

```text
USAGE:
    forc-index remove [OPTIONS] --manifest <MANIFEST>

OPTIONS:
        --auth <AUTH>            Authentication header value.
    -h, --help                   Print help information
        --manifest <MANIFEST>    Path of the index manifest to be parsed.
        --url <URL>              URL at which to upload index assets. [default:
                                 http://127.0.0.1:29987]
```

### `forc index check`

Check to see which indexer components you have installed

```bash
forc index check
```

```text
USAGE:
    forc-index check [OPTIONS]

OPTIONS:
        --grpahql-api-port <GRPAHQL_API_PORT>
            Port at which to detect indexer service API is running. [default: 29987]

    -h, --help
            Print help information

        --url <URL>
            URL at which to find indexer service. [default: http://127.0.0.1:29987]
```

### `forc index build`

Build an index

```bash
forc index build --release --manifest my_index.manifest.yaml
```

```text
USAGE:
    forc-index build [OPTIONS] --manifest <MANIFEST>

OPTIONS:
    -h, --help                   Print help information
        --locked                 Ensure that the Cargo.lock file is up-to-date.
    -m, --manifest <MANIFEST>    Path of index manifest being built.
        --native                 Building for native execution.
        --profile <PROFILE>      Build with the given profile.
    -r, --release                Build optimized artifacts with the release profile.
        --target <TARGET>        Target at which to compile.
    -v, --verbose                Verbose output.
```
