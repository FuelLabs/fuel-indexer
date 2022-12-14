#!/bin/bash

set -x

docker build \
    --no-cache \
    -t local/fuel-indexer-docker-env:latest \
    -f deployment/Dockerfile .

cd scripts/

docker-compose -f docker-compose.yaml up
