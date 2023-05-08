#!/bin/bash

# Check if command exists
if !(command -v "forc-index" >/dev/null 2>&1) ; then
    echo "'forc index' is not installed. https://install.fuel.network/latest"
    exit 1;
fi

pnpm exec nodemon --config ./nodemon.json