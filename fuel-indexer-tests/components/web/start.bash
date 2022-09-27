#!/bin/bash

set -e

./src/bin/fuel-node \
    --wallet-path $PWD/wallet.json \
    --bin-path $PWD/../../contracts/fuel-indexer/out/debug/fuel-indexer.bin &

./src/bin/web-api \
    --wallet-path $PWD/wallet.json \
    --bin-path $PWD/../../contracts/fuel-indexer/out/debug/fuel-indexer.bin &
