#!/bin/bash
#
# Drops the test database, recreates it, and runs migrations
#
# Usage:
#   bash scripts/utils/refresh_test_db.bash

set -x

db_arg="${1}"
db_type=${db_arg:=postgres}

if [ $db_type == "postgres" ]; then
    dropdb postgres
    createdb postgres
    DATABASE_URL=postgres://postgres@127.0.0.1 bash scripts/run_migrations.bash
else
    echo "Invalid db param. Expected 'postgres'. Found '$db_type'"
    exit 1
fi

set +x
