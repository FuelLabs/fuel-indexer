#!/bin/bash

echo """
namespace: composability_test
graphql_schema: tests/e2e/composable-indexer/composable-indexer-lib/schema/schema.graphql
wasm_module: tests/e2e/composable-indexer/composable-indexer-lib/composable_indexer_lib.wasm
handlers:
  - event: LogData
    handler: function_one
""" > manifest.yaml
