# `forc index build`

Build an indexer.

```bash
forc index build --release
```

```text
USAGE:
    forc-index build [OPTIONS]

OPTIONS:
    -h, --help
            Print help information

        --locked
            Ensure that the Cargo.lock file is up-to-date.

    -m, --manifest <MANIFEST>
            Manifest file name of indexer being built.

        --native
            Building for native execution.

        --output-dir-root <OUTPUT_DIR_ROOT>
            Path with which to prefix asset filepaths in the indexer manifest.

    -p, --path <PATH>
            Path to the indexer project.

        --profile <PROFILE>
            Build with the given profile.

    -r, --release
            Build optimized artifacts with the release profile.

        --target <TARGET>
            Target at which to compile.

    -v, --verbose
            Verbose output.
```
