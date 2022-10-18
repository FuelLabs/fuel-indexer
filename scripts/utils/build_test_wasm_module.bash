#!/bin/bash
#
# Usage: bash scripts/build_test_wasm_module.bash

set -ex

cargo build -p fuel-indexer-test --release --target wasm32-unknown-unknown
bash scripts/stripper.bash fuel_indexer_test.wasm
cp fuel_indexer_test.wasm fuel-indexer-tests/assets/
rm -fv fuel_indexer_test.wasm

set +ex
