# `fuel-indexer-tests/trybuild/abi`

- This directory contains several Sway JSON ABI files copied over from various [Sway applications](https://github.com/FuelLabs/sway-applications/tree/master).
- We use this ABI in a few trybuild tests in order to ensure that `forc index` builds with actual/legitimate Sway code (not just indexer-related test code)
- We include these JSON ABI files here (in their own `abi` directory) because we are only using ABI files in the relevant tests (i.e., we are not combining these ABI files with GraphQL schema).