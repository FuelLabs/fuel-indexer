# `forc index build`

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
