#!/bin/bash

# This query cleans alls tables from the indexer_indexer schema

echo "🔄 Restart fuel-explorer-indexer container"

pnpm -w services:stop
pnpm -w services:start

echo "⚠️ Restart the 'pnpm dev:indexer' to start indexing"