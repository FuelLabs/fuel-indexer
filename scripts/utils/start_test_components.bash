#!/bin/bash
# 
# Start the web components used by fuel-indexer-tests. Expects binaries for test 
# components to have already been built and up-to-date via the
# scripts/utils/test_binaries_checksums.bash script.
#
# Usage: 
#   bash scripts/utils/start_test_components.bash
#

set -x

bash ./scripts/utils/kill_test_components.bash

sleep 1

./fuel-indexer-tests/assets/bin/fuel-node \
    --wallet-path ./fuel-indexer-tests/components/web/wallet.json \
    --bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &

sleep 1

./fuel-indexer-tests/assets/bin/web-api \
    --wallet-path ./fuel-indexer-tests/components/web/wallet.json \
    --bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &

if [[ ! -z $CI ]] ; then
    sleep 2

    ./fuel-indexer-tests/assets/bin/fuel-indexer \
        --manifest fuel-indexer-tests/assets/fuel_indexer_test.yaml \
        --postgres-password my-secret &
fi

set +x
