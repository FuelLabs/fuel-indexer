#!/bin/bash
#
# Rebuilds all WASM modules used for testing
#
#   Usage: bash scripts/utils/build_test_wasm_module.bash

set -ex

# This is a test index, keep it in assets for now
cargo build -p fuel-indexer-test --release --target wasm32-unknown-unknown
bash scripts/stripper.bash fuel_indexer_test.wasm
cp fuel_indexer_test.wasm target/wasm32-unknown-unknown/release/
rm -fv fuel_indexer_test.wasm

# This is a test index, keep it in assets for now
cargo build -p simple-wasm --release --target wasm32-unknown-unknown
bash scripts/stripper.bash simple_wasm.wasm
cp simple_wasm.wasm target/wasm32-unknown-unknown/release/
cp simple_wasm.wasm packages/fuel-indexer-tests/components/indices/simple-wasm
rm -fv simple_wasm.wasm

cargo build -p explorer_indexer --release --target wasm32-unknown-unknown
bash scripts/stripper.bash explorer_indexer.wasm
cp explorer_indexer.wasm target/wasm32-unknown-unknown/release/
rm -fv explorer_indexer.wasm

cargo build -p hello_indexer --release --target wasm32-unknown-unknown
bash scripts/stripper.bash hello_indexer.wasm
cp hello_indexer.wasm target/wasm32-unknown-unknown/release/
rm -fv hello_indexer.wasm

cargo build -p two-contracts --release --target wasm32-unknown-unknown
bash scripts/stripper.bash two_contracts.wasm
cp two_contracts.wasm target/wasm32-unknown-unknown/release/
rm -fv two_contracts.wasm

set +ex
