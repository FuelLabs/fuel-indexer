#!/bin/bash
#
# Ensures that component binaries used for testing are the up to date. This ensures
# that we don't have to build fuel-indexer-test-web on CI thus cutting down significantly
# on CI time. This is a temporary measure until https://github.com/FuelLabs/fuel-indexer/issues/269
#
# Usage:
#
#    To validate checksums:
#         bash scripts/utils/validate_test_binaries.bash
#
#     To create new binaries (if checksums don't match):
#         bash scripts/utils/validate_test_binaries.bash $(which md5sum) build
#

# Use a default of sha1sum which is decently portable. But allow the user to pass
# a custom hash function if sha1sum is unavailable for them.
hash_fn="${1:-sha1sum}"
op=$2

if [[ $op = 'build' ]]; then

    cargo build -p fuel-indexer-test-web -p fuel-indexer --release
    cp target/release/fuel-node* fuel-indexer-tests/assets/bin
    cp target/release/web-api* fuel-indexer-tests/assets/bin
    cp target/release/fuel-indexer* fuel-indexer-tests/assets/bin

else

    target_web_api_checksum=$(cat target/release/web-api | base64 | $hash_fn)
    target_fuel_node_checksum=$(cat target/release/fuel-node | base64 | $hash_fn)
    target_indexer_checksum=$(cat target/release/fuel-indexer | base64 | $hash_fn)

    printf "\ntarget/release/web-api checksum: $target_web_api_checksum\n"
    printf "target/release/fuel-node checksum: $target_fuel_node_checksum\n"
    printf "target/release/fuel-indexer checksum: $target_indexer_checksum\n"

    test_web_api_checksum=$(cat fuel-indexer-tests/assets/bin/web-api | base64 | $hash_fn)
    test_fuel_node_checksum=$(cat fuel-indexer-tests/assets/bin/fuel-node | base64 | $hash_fn)
    test_indexer_checksum=$(cat fuel-indexer-tests/assets/bin/fuel-indexer | base64 | $hash_fn)

    printf "\nfuel-indexer-tests/assets/bin/web-api checksum: $test_web_api_checksum\n"
    printf "fuel-indexer-tests/assets/bin/fuel-node  checksum: $test_fuel_node_checksum\n"
     printf "fuel-indexer-tests/assets/bin/fuel-indexer  checksum: $test_indexer_checksum\n"

    if [[ $target_web_api_checksum != $test_web_api_checksum ]]; then
        printf "\nChecksums for web-api binary at target/release/web-api and fuel-indexer-tests/assets/bin/web-api do not match. <('-'<)\n"
        printf "\nPlease run 'bash scripts/utils/validate_test_binaries.bash $hash_fn build' to fix.\n\n"
        exit 1
    fi

    if [[ $target_fuel_node_checksum != $test_fuel_node_checksum ]]; then
        printf "\nChecksums for fuel-node binary at target/release/fuel-node and fuel-indexer-tests/assets/bin/fuel-node do not match. <('-'<)\n"
        printf "\nPlease run 'bash scripts/utils/validate_test_binaries.bash $hash_fn build' to fix.\n\n"
        exit 1
    fi

    if [[ $target_indexer_checksum != $test_indexer_checksum ]]; then
        printf "\nChecksums for fuel-indexer binary at target/release/fuel-indexer and fuel-indexer-tests/assets/bin/fuel-indexer do not match. <('-'<)\n"
        printf "\nPlease run 'bash scripts/utils/validate_test_binaries.bash $hash_fn build' to fix.\n\n"
        exit 1
    fi

    printf "\nAll checksums match. <(^_^)>\n\n"
fi
