#!/bin/bash
SQLITE_URL=sqlite://${PWD}/fuel-indexer/test.db

export DATABASE_URL=postgres://postgres:my-secret@localhost:5432/postgres
cd database/postgres;
sqlx migrate run

cd ../sqlite
export DATABASE_URL=$SQLITE_URL
sqlx database create
sqlx migrate run
