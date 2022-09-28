#!/bin/bash

set -e

./src/bin/fuel-node \
    --wallet-path $PWD/wallet.json \
    --bin-path $PWD/../../contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &

./src/bin/web-api \
    --wallet-path $PWD/wallet.json \
    --bin-path $PWD/../../contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &
