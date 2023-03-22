# `forc index deploy`

Deploy an indexer to an indexer service.

```bash
forc index deploy --url https://indexer.fuel.network
```

```text
USAGE:
    forc-index deploy [OPTIONS]

OPTIONS:
        --auth <AUTH>
            Authentication header value.

    -h, --help
            Print help information

        --locked
            Ensure that the Cargo.lock file is up-to-date.

    -m, --manifest <MANIFEST>
            Path to the manifest of indexer project being deployed.

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

        --url <URL>
            URL at which to deploy indexer assets. [default: http://127.0.0.1:29987]

    -v, --verbose
            Verbose output.

```
