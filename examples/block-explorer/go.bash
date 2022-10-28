#!/bin/bash

while true; do
    echo "Generating some blocks... <('.'<)"
    curl -X POST http://127.0.0.1:8000/block
    sleep 1
done
