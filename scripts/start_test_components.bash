#!/bin/bash

bash ./scripts/kill_test_components.bash

./target/release/fuel-node \
    --wallet-path ./fuel-indexer-tests/components/web/wallet.json \
    --bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &

./target/release/web-api \
    --wallet-path ./fuel-indexer-tests/components/web/wallet.json \
    --bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &
