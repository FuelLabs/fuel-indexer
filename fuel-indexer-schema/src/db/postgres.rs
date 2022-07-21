mod schema;

pub use schema::*;

use diesel_migrations::embed_migrations;

embed_migrations!("migrations/postgres");

pub fn run_migration(conn: &diesel::pg::PgConnection) {
    embedded_migrations::run(conn).expect("Could not run postgres migrations!");
}
