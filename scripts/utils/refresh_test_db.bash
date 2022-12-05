#!/bin/bash
#
# Drops the test database, recreates it, and runs migrations
#
# Usage:
#   bash scripts/utils/refresh_test_db.bash

set -x

db_type="${1}"

if [ $db_type == "postgres" ]; then
    dropdb postgres
    createdb postgres
    DATABASE_URL=postgres://postgres@127.0.0.1 bash scripts/run_migrations.bash
elif [ $db_type == "sqlite" ]; then
    rm -fv $PWD/packages/fuel-indexer-tests/test.db
    DATABASE_URL=sqlite://$PWD/packages/fuel-indexer-tests/test.db bash scripts/run_migrations.bash
else
    echo "Invalid db param. Expected 'sqlite' or 'postgres'. Found '$db_type'"
    exit 1
fi

set +x
