#!/bin/bash
#
# Usage: bash scripts/utils/kill_test_components.bash

for port in 4000 8000 29987; do
    proc=$(lsof -ti:$port)
    if [[ ! -z $proc ]]; then
        echo "Killing process at: $proc"
        kill -9 $proc
    else
        echo "No process at port: $port"
    fi
done
