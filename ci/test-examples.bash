#!/bin/bash

# This script is intended to ensure that our examples build, deploy, and actually
# index data. The intention is that via testing this from the user perspective
# (rather than the dev perspective) we can catch any issues on CI, without having
# to write and maintain copious amounts of integration/unit tests for an API that
# is still in flux.

set -ex

# ********************************
# examples/hello-indexer
# ********************************

# Go to example
cd examples/hello-world/

# Start the local Fuel node in the background
cargo build -p hello-world-node
cargo run -p hello-world-node --bin hello-world-node &
sleep 2

# Start service in the background
forc-index start --postgres-password my-secret --run-migrations &
sleep 2

# Deploy the example indexer
forc-index deploy --path ./hello-indexer
sleep 5

# Ensure service is up and running via health check
db_status=$(curl http://localhost:29987/api/health | json_pp | jq '.database_status')
sleep 1
client_status=$(curl http://localhost:29987/api/health | json_pp | jq '.client_status')

if [[ "$db_status" == "$client_status" ]] && [[ "$db_status" == '"OK"' ]]; then
  echo "Database and client are up and running."
else
  echo "Database and/or client are not up and running."
  exit 1
fi


# Trigger an indexable event
cargo run -p hello-world-data --bin hello-world-data
sleep 2

# Check data is indexed
export PGPASSWORD=my-secret
echo "SELECT COUNT(*) FROM fuel_examples.salutation;" | psql -h localhost -p 5432 -U postgres
sleep 2

# Shutdown the service
forc-index kill
sleep 2

# ********************************
# examples/hello-indexer-native
# ********************************

# TODO: https://github.com/FuelLabs/fuel-indexer/issues/1069

cargo build -p hello_indexer_native --locked --release

# ********************************
# examples/fuel-explorer
# ********************************

# Go to example
cd ../fuel-explorer

# Start service in the background
forc-index start --postgres-password my-secret --run-migrations &
sleep 2

# Deploy the example indexer
forc-index deploy --path ./fuel-explorer
sleep 5

# Ensure service is up and running via 
db_status=$(curl http://localhost:29987/api/health | json_pp | jq '.database_status')
sleep 1
client_status=$(curl http://localhost:29987/api/health | json_pp | jq '.client_status')

if [[ "$db_status" == "$client_status" ]] && [[ "$db_status" == '"OK"' ]]; then
  echo "Database and client are up and running."
else
  echo "Database and/or client are not up and running."
exit 1
fi

# Check data is indexed
export PGPASSWORD=my-secret
echo "SELECT COUNT(*) FROM fuel_explorer.block;" | psql -h localhost -p 5432 -U postgres
sleep 2

# Shutdown the service
forc-index kill
sleep 2

# Finally, kill the hello-world-node
kill -9 $(lsof -ti:4000)
