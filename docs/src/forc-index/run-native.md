# `forc index run-native`

Run a native indexer.

```bash
forc index run-native --path . -- --run-migrations --postgres-host localhost
```

```text
Run a native indexer

USAGE:
    forc-index run-native [OPTIONS] [-- <ARGS>...]

ARGS:
    <ARGS>...
            Extra passed to `fuel-indexer run`

            Example usage: `forc-index run-native --path . -- --run-migrations --stop-idle-indexers`

OPTIONS:
        --bin <BIN>
            Path to native indexer binary (if not using default location).

    -d, --debug
            Build artifacts with the debug profile.

    -h, --help
            Print help information

        --locked
            Ensure that the Cargo.lock file is up-to-date.

    -m, --manifest <MANIFEST>
            Manifest file name of indexer being built.

    -p, --path <PATH>
            Path to the indexer project.

        --skip-build
            Do not build before deploying.

    -v, --verbose
            Enable verbose output.
```
