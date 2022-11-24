#!/bin/bash

while true; do
    echo "Generating transfers... <('.'<)"
    curl -X POST http://127.0.0.1:8000/preload_transfers
    sleep 2
done
