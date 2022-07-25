pub mod graphql;
pub mod models;
pub mod tables;
use sqlx::pool::PoolConnection;


#[derive(Clone)]
pub enum IndexerConnectionPool {
    Postgres(sqlx::Pool<sqlx::Postgres>),
    Sqlite(sqlx::Pool<sqlx::Sqlite>),
}

impl IndexerConnectionPool {
    pub async fn connect(database_url: &str) -> IndexerConnectionPool {
        let url = url::Url::parse(database_url);
        if url.is_err() {
            panic!("{} is not a valid database string!", database_url);
        }
        let url = url.unwrap();

        match url.scheme() {
            "postgres" => {
                let pool = sqlx::Pool::<sqlx::Postgres>::connect(database_url).await
                    .expect("Something went wrong establishing a pg connection!");
                IndexerConnectionPool::Postgres(pool)
            }
            "sqlite" => {
                let pool = sqlx::Pool::<sqlx::Sqlite>::connect(database_url).await
                    .expect("Something went wrong establishing a pg connection!");
                IndexerConnectionPool::Sqlite(pool)
            }
            e => {
                panic!("database {} is not supported, use sqlite or postgres", e);
            }
        }
        
    }

    pub async fn acquire(&self) -> sqlx::Result<IndexerConnection> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                Ok(IndexerConnection::Postgres(p.acquire().await?))
            }
            IndexerConnectionPool::Sqlite(p) => {
                Ok(IndexerConnection::Sqlite(p.acquire().await?))
            }
        }
    }
}

pub enum IndexerConnection {
    Postgres(PoolConnection<sqlx::Postgres>),
    Sqlite(PoolConnection<sqlx::Sqlite>),
}

use fuel_indexer_sqlite as sqlite;
use fuel_indexer_postgres as postgres;

pub async fn run_migration(database_url: &str) {
    let url = url::Url::parse(database_url);
    if url.is_err() {
        panic!("{} is not a valid database string!", database_url);
    }
    let url = url.unwrap();

    match url.scheme() {
        "postgres" => {
            postgres::run_migration(database_url).await;
        }
        "sqlite" => {
            sqlite::run_migration(database_url).await;
        }
        e => {
            panic!("database {} is not supported, use sqlite or postgres", e);
        }
    }
}
