#!/bin/bash

set -e  # exit if any command fails

bash scripts/utils/refresh_test_db.bash
bash scripts/utils/build_test_wasm_module.bash
bash scripts/utils/run_tests.bash
