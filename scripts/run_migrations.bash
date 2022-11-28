#!/bin/bash

set -ex

if [[ -z $CI ]] ; then
    if [ -z "$DATABASE_URL" ]; then
        echo "DATABASE_URL is not set in this shell. Please set DATABASE_URL and retry."
        exit 1
    else
        echo "Migrating using DATABASE_URL at '$DATABASE_URL'"
    fi

    if [[ $DATABASE_URL = postgres* ]]; then
        cd packages/fuel-indexer-database/postgres
        DATABASE_URL=$DATABASE_URL sqlx migrate run
    elif [[ $DATABASE_URL = sqlite* ]]; then
        cd packages/fuel-indexer-database/sqlite
        DATABASE_URL=$DATABASE_URL sqlx database create
        DATABASE_URL=$DATABASE_URL sqlx migrate run
    else
        echo "Unrecognized database path prefix on DATABASE_URL '$DATABASE_URL'"
        exit 1
    fi
else
    export DATABASE_URL="postgres://postgres:my-secret@localhost:5432/postgres"

    cd packages/fuel-indexer-database/postgres
    sqlx migrate run

    cd -

    touch $PWD/packages/fuel-indexer-tests/test.db
    export DATABASE_URL="sqlite://${PWD}/packages/fuel-indexer-tests/test.db"
    cd packages/fuel-indexer-database/sqlite
    sqlx database create
    sqlx migrate run
fi

set +ex
