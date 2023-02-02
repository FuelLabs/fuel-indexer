#!/bin/bash

set -ex

cd packages/

if [[ -z $CI ]] ; then
    if [ -z "$DATABASE_URL" ]; then
        echo "DATABASE_URL is not set in this shell. Please set DATABASE_URL and retry."
        exit 1
    else
        echo "Migrating using DATABASE_URL at '$DATABASE_URL'"
    fi

    if [[ $DATABASE_URL = postgres* ]]; then
        cd fuel-indexer-database/postgres
        DATABASE_URL=$DATABASE_URL sqlx migrate run
    else
        echo "Unrecognized database path prefix on DATABASE_URL '$DATABASE_URL'"
        exit 1
    fi
else
    export DATABASE_URL="postgres://postgres:my-secret@localhost:5432/postgres"

    cd fuel-indexer-database/postgres
    sqlx migrate run

    cd -
fi

cd ..

set +ex
