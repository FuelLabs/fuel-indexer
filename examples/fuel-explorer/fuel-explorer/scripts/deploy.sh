#!/bin/bash

# Check if command exists
if !(command -v "forc-index" >/dev/null 2>&1) ; then
    echo "'forc index' is not installed. https://install.fuel.network/latest"
    exit 1;
fi

forc index deploy --url http://0.0.0.0:29987