# `forc index deploy`

Deploy an indexer to an indexer service.

```bash
forc index deploy --url https://indexer.fuel.network
```

```text
USAGE:
    forc-index deploy [OPTIONS]

OPTIONS:
        --auth <AUTH>                Authentication header value.
    -h, --help                       Print help information
        --locked                     Ensure that the Cargo.lock file is up-to-date.
    -m, --manifest <MANIFEST>        Path to the manifest of indexer project being deployed.
        --native                     Building for native execution.
    -p, --path <PATH>                Path to the indexer project.
        --profile <PROFILE>          Build with the given profile.
    -r, --release                    Build optimized artifacts with the release profile.
        --skip-build                 Do not build before deploying.
        --target <TARGET>            Target at which to compile. [default: wasm32-unknown-unknown]
        --target-dir <TARGET_DIR>    Directory for all generated artifacts and intermediate files.
        --url <URL>                  URL at which to deploy indexer assets. [default:
                                     http://localhost:29987]
    -v, --verbose                    Enable verbose logging.
```
