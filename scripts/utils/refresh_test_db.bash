#!/bin/bash
#
# Drops the test database, recreates it
#
# Usage:
#   bash scripts/utils/refresh_test_db.bash

set -x

db=${1:-postgres}
run_migrations=${2:-false}

if [ $db == "postgres" ]; then
    dropdb postgres
    createdb postgres
    if [ $run_migrations == "true" ]; then
        DATABASE_URL=postgres://postgres@localhost bash scripts/run_migrations.bash
    fi
else
    echo "Invalid db param. Expected 'postgres'. Found '$db_type'"
    exit 1
fi

set +x
