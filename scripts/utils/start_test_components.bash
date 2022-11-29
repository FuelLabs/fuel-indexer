#!/bin/bash
#
# Usage: bash scripts/utils/start_test_components.bash
#
# Expects binaries for test components to have already been built via:
#   cargo build -p fuel-indexer-test-web --release

set -x

bash ./scripts/utils/kill_test_components.bash

export CARGO_MANIFEST_DIR=$PWD/fuel-indexer-tests/components/web

sleep 1

./target/release/fuel-node \
<<<<<<< HEAD
    --wallet-path ./fuel-indexer-tests/assets/test-chain-config.json \
    --contract-bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &
=======
    --wallet-path packages/fuel-indexer-tests/components/web/wallet.json \
    --bin-path packages/fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &
>>>>>>> a0ab2354c5936a99fc9f00593e4e1d143a7b5097

sleep 1

./target/release/web-api \
<<<<<<< HEAD
    --wallet-path ./fuel-indexer-tests/assets/test-chain-config.json \
    --contract-bin-path ./fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &
=======
    --wallet-path packages/fuel-indexer-tests/components/web/wallet.json \
    --bin-path packages/fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin &
>>>>>>> a0ab2354c5936a99fc9f00593e4e1d143a7b5097

if [[ ! -z $CI ]] ; then
    sleep 2

    ./target/release/fuel-indexer \
        --manifest packages/fuel-indexer-tests/assets/fuel_indexer_test.yaml \
        --postgres-password my-secret &
fi

set +x
