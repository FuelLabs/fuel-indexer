#!/bin/bash

set -x

docker build \
    --no-cache \
    -t local/fuel-indexer-docker-env:latest \
    -f docker/Dockerfile.x86_64-unknown-linux-gnu .

cd docker/

docker-compose -f docker-compose.x86_64-unknown-linux-gnu.yaml up
