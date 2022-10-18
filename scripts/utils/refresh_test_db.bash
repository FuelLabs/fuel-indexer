#!/bin/bash
#
# Drops the test database, recreates it, and runs migrations
#
# Usage:
#   bash scripts/utils/refresh_test_db.bash

set -x

dropdb postgres
createdb postgres
DATABASE_URL=postgres://postgres@127.0.0.1 bash scripts/run_migrations.bash

set +x
