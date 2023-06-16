# `forc index check`

Check to see which indexer components you have installed.

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
            URL at which to find indexer service. [default: http://localhost:29987]
```

You can expect the command output to look something like this example in which the requisite components are installed but the indexer service is not running:

```text
➜  forc index check

❌ Could not connect to indexer service: error sending request for url (http://localhost:29987/api/health): error trying to connect: tcp connect error: Connection refused (os error 61)

+--------+------------------------+----------------------------------------------------------------------------+
| Status |       Component        |                                  Details                                   |
+--------+------------------------+----------------------------------------------------------------------------+
|   ✅   | fuel-indexer binary    |  Found 'fuel-indexer' at '/Users/me/.fuelup/bin/fuel-indexer'              |
+--------+------------------------+----------------------------------------------------------------------------+
|   ⛔️   | fuel-indexer service   |  Failed to detect a locally running fuel-indexer service at Port(29987).   |
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
