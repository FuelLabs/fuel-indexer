# `forc index check`

Check to see which indexer components you have installed.

```bash
forc index check
```

```text
USAGE:
    forc-index check

OPTIONS:
    -h, --help
            Print help information
```

You can expect the command output to look something like this:

```text
➜  forc index check

+--------+------------------------+----------------------------------------------------------------------------+
| Status |       Component        |                                  Details                                   |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuel-indexer binary    |  Found 'fuel-indexer' at '/Users/me/.fuelup/bin/fuel-indexer'              |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | psql                   |  Found 'psql' at '/usr/local/bin/psql'                                     |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuel-core              |  Found 'fuel-core' at '/Users/me/.fuelup/bin/fuel-core'                    |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | docker                 |  Found 'docker' at '/usr/local/bin/docker'                                 |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuelup                 |  Found 'fuelup' at '/Users/me/.fuelup/bin/fuelup'                          |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | wasm-snip              |  Found 'wasm-snip' at '/Users/me/.cargo/bin/wasm-snip'                     |
+--------+------------------------+----------------------------------------------------------------------------+
```
