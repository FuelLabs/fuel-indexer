# `forc index build`

Build an indexer.

```bash
forc index build --release
```

```text
USAGE:
    forc-index build [OPTIONS]

OPTIONS:
    -h, --help                       Print help information
        --locked                     Ensure that the Cargo.lock file is up-to-date.
    -m, --manifest <MANIFEST>        Manifest file name of indexer being built.
        --native                     Building for native execution.
    -p, --path <PATH>                Path to the indexer project.
        --profile <PROFILE>          Build with the given profile.
    -r, --release                    Build optimized artifacts with the release profile. Set to true by default.
        --target <TARGET>            Target at which to compile. [default: wasm32-unknown-unknown]
        --target-dir <TARGET_DIR>    Directory for all generated artifacts and intermediate files.
    -v, --verbose                    Enable verbose output.

```
