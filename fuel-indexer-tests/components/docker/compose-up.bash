#!/bin/bash

set -exo

docker build \
    --no-cache \
    -t local/composable-indexer-test:latest \
    -f tests/e2e/composable-indexer/Dockerfile .


cd tests/e2e/composable-indexer

docker-compose up
