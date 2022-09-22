
export DATABASE_URL="postgres://postgres:my-secret@localhost:5432/postgres"

cd fuel-indexer-database/postgres
sqlx migrate run

cd -

export DATABASE_URL="sqlite://${PWD}/fuel-indexer/test.db"
cd fuel-indexer-database/sqlite
sqlx database create
sqlx migrate run
