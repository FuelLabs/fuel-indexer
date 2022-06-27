use diesel::Connection;

pub mod graphql;
pub mod models;
#[allow(unused_imports)]
pub mod postgres;
pub mod tables;

pub fn run_migration(database_url: &String) {
    let conn = diesel::pg::PgConnection::establish(database_url)
        .expect("Could not establish pg connection");
    postgres::run_migration(&conn);
}
