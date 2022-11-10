#!/bin/bash
#
# Ensures that component binaries used for testing are the up to date. This ensures
# that we don't have to build fuel-indexer-test-web on CI thus cutting down significantly
# on CI time. This is a temporary measure until https://github.com/FuelLabs/fuel-indexer/issues/269
#
# Usage: 
#     
#    To validate checksums:
#         bash scripts/utils/test_binaries_checksums.bash $(which md5sum)
# 
#     To create new binaries (if checksums don't match):
#         bash scripts/utils/test_binaries_checksums.bash $(which md5sum) build
#

hasher=$1
op=$2

if [[ $op = 'build' ]]; then

    cargo build -p fuel-indexer-test-web --release
    cp target/release/fuel-node* fuel-indexer-tests/components/web/src/bin
    cp target/release/web-api* fuel-indexer-tests/components/web/src/bin

else

    target_web_api_checksum=$(cat target/release/web-api | base64 | $hasher)
    target_fuel_node_checksum=$(cat target/release/fuel-node | base64 | $hasher)

    echo "target/release/web-api checksum: $target_web_api_checksum\n"
    echo "target/release/fuel-node checksum: $target_fuel_node_checksum\n"


    test_web_api_checksum=$(cat fuel-indexer-tests/components/web/src/bin/web-api | base64 | $hasher)
    test_fuel_node_checksum=$(cat fuel-indexer-tests/components/web/src/bin/fuel-node | base64 | $hasher)

    echo "fuel-indexer-tests/components/web/src/bin/web-api checksum: $test_web_api_checksum\n"
    echo "fuel-indexer-tests/components/web/src/bin/fuel-node  checksum: $test_fuel_node_checksum\n"

    if [[ $target_web_api_checksum != $test_web_api_checksum ]]; then
        echo "Checksums for web-api binary at target/release/web-api and fuel-indexer-tests/components/web/src/bin/web-api do not match. <('-'<)"
        echo "Please run bash scripts/utils/test_binaries_checksums.bash build to fix."
        exit 1
    fi

    if [[ $target_fuel_node_checksum != $test_fuel_node_checksum ]]; then
        echo "Checksums for fuel-node binary at target/release/fuel-node and fuel-indexer-tests/components/web/src/bin/fuel-node do not match. <('-'<)"
        echo "Please run bash scripts/utils/test_binaries_checksums.bash build to fix."
        exit 1
    fi

    echo "All checksums match. <(^_^)>"
fi
