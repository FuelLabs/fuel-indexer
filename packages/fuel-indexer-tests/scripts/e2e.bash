#!/bin/bash

# NOTE: There's 2 categories of tests on the indexer (so far): end-to-end
# (e2e) tests, and non-end-to-end tests. E2E tests are some of the most
# comprehensive tests we have, in that they test the integration between
# _all_ indexer components - database, Fuel node, transactions, executors,
# etc. E2E tests when run with default `cargo test` produce inconsistent/flaky
# functionality, even when run with `--test-threads 1`. This e2e.bash file was
# created in order to test each of these e2e tests in isolation - with no other
# tests running, for consistent results.  Again, this is limited to _only_ e2e
# tests - all other tests work fine using default `cargo test` behavior. Running
# each test solely - in isolation takes about 3 mins longer than running the
# tests together.

cargomodules=(
    packages/fuel-indexer-tests/tests/e2e/indexing_postgres.rs
)

for module in "${cargomodules[@]}"
do
    tests=(`grep -r 'test_.*' $module | awk '{ print substr($3, 0, length($3)-2) }' | grep -E 'test_*' | awk -F'\\\n' '{print $1}'`)

    for test in "${tests[@]}"
    do
        cargo test $test --locked --features e2e,postgres --all-targets
    done
done


cargo test test_can_trigger_and_index_log_event_postgres --locked --features e2e,postgres --all-targets
cargo test test_can_trigger_and_index_logdata_event_postgres --locked --features e2e,postgres --all-targets