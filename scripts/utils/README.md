# scripts/utils

General utilty scripts used to improve devx

```text
.
├── README.md
├── build_test_wasm_module.bash
├── kill_test_components.bash
├── refresh_test_db.bash
└── start_test_components.bash

0 directories, 5 files
```

- build_test_wasm_module
  - Build the default `fuel-indexer-test` WASM module and add it to `fuel-indexer-tests/assets`
- kill_test_components
  - Kill all processes for test components (Fuel node, Web API)
- refresh_test_db
  - Drop the default testing database, recreate it, and run migrations
- start_test_components
  - Start all testing components (Fuel node, Web API)
