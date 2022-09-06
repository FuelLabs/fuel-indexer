#!/bin/bash

echo """
namespace: composability_test
graphql_schema: composable-indexer-lib/schema/schema.graphql
module:
  wasm:
    composable-indexer-lib/composable_indexer_lib.wasm
""" > manifest.yaml
