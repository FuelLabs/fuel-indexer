#!/bin/bash

bash ./scripts/kill_test_components.bash

./target/release/fuel-node \
    --wallet-path ./fuel-indexer-tests/components/web/wallet.json \
    --bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &

sleep 2

./target/release/web-api \
    --wallet-path ./fuel-indexer-tests/components/web/wallet.json \
    --bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &

sleep 2

./target/release/fuel-indexer \
    --manifest fuel-indexer-tests/assets/fuel_indexer_test.yaml \
    --postgres-password my-secret &
