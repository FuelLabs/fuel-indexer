#!/bin/bash

# This script is intended to ensure that our examples build, deploy, and actually
# index data. The intention is that via testing this from the user perspective
# (rather than the dev perspective) we can catch any issues on CI, without having
# to write and maintain copious amounts of integration/unit tests for an API that
# is still in flux.


check_service_status() {
  db_status=$(curl -s http://localhost:29987/api/health | jq '.database_status')
  sleep 1
  client_status=$(curl -s http://localhost:29987/api/health | jq '.client_status')

  if [[ "$db_status" == "$client_status" ]] && [[ "$db_status" == '"OK"' ]]; then
    echo "Database and client are up and running."
  else
    echo "Database and/or client are not up and running."
    exit 1
  fi
}

export PGPASSWORD=my-secret

set -ex

forc_index_kill() {
  # Shutdown the service
  forc-index kill
  sleep 2
}

# ********************************
# examples/greetings
# ********************************

cd examples/greetings/

cargo build -p greetings-fuel-client
cargo run -p greetings-fuel-client --bin greetings-fuel-client &
sleep 1

forc-index start --postgres-password my-secret --run-migrations &
sleep 1

forc-index deploy --path ./greetings-indexer
sleep 3

# check_service_status

# Trigger an indexable event
cargo run -p greetings-data --bin greetings-data
sleep 1

# Check data is indexed
echo "SELECT COUNT(*) FROM fuellabs_greetings.salutation;" | psql -h localhost -p 5432 -U postgres
sleep 2

forc_index_kill

# ********************************
# examples/greetings-native
# ********************************

# cd ../greetings-native

# forc-index build --path ./greetings-native-indexer --native

# # Update manifest with paths relative to the example due to cargo workspace
# sed -i \
#   "s|^graphql_schema: examples/greetings-native/greetings-native-indexer/schema/greetings_native_indexer\.schema\.graphql|graphql_schema: $PWD/greetings-native-indexer/schema/greetings_native_indexer.schema.graphql|" \
#     ./greetings-native-indexer/greetings_native_indexer.manifest.yaml

# Start indexer in the background
# forc-index run-native --path ./greetings-native-indexer -- --run-migrations --postgres-password my-secret
# sleep 60

# check_service_status

# cargo run -p greetings-data --bin greetings-data
# sleep 5

# # Check data is indexed
# echo "SELECT COUNT(*) FROM fuellabs_greetings_native.salutation;" | psql -h localhost -p 5432 -U postgres
# sleep 2

# forc_index_kill

# ********************************
# examples/fuel-explorer
# ********************************

# Go to example
cd ../fuel-explorer

# Start service in the background
forc-index start --postgres-password my-secret --run-migrations &
sleep 2

forc-index deploy --path ./fuel-explorer
sleep 5

check_service_status

echo "SELECT COUNT(*) FROM fuellabs_explorer.block;" | psql -h localhost -p 5432 -U postgres
sleep 2

forc_index_kill

# ********************************
# examples/hello-world
# ********************************

# Go to example
cd ../hello-world

# Start service in the background
forc-index start --postgres-password my-secret --run-migrations &
sleep 2

# Deploy the example indexer
forc-index deploy --path ./hello-world
sleep 5

check_service_status

# Check data is indexed
echo "SELECT COUNT(*) FROM fuellabs_hello_world.block;" | psql -h localhost -p 5432 -U postgres
sleep 2

forc_index_kill

# Finally, kill the greetings-fuel-client
kill -9 $(lsof -ti:4000)
