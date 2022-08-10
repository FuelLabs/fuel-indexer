pub mod graphql;
pub mod models;
pub mod tables;
use sqlx::pool::PoolConnection;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum DatabaseError {
    #[error("Invalid connection string: {0:?}")]
    InvalidConnectionString(String),
    #[error("Database backend not supported: {0:?}")]
    BackendNotSupported(String),
}

#[derive(Clone, Debug)]
pub enum IndexerConnectionPool {
    Postgres(sqlx::Pool<sqlx::Postgres>),
    Sqlite(sqlx::Pool<sqlx::Sqlite>),
}

#[derive(Default, PartialEq)]
pub enum DbType {
    #[default]
    Postgres,
    Sqlite,
}

impl DbType {
    pub fn table_name(&self, namespace: &str, table_name: &str) -> String {
        match self {
            DbType::Postgres => format!("{}.{}", namespace, table_name),
            DbType::Sqlite => table_name.to_string(),
        }
    }
}

impl IndexerConnectionPool {
    pub fn database_type(&self) -> DbType {
        match self {
            IndexerConnectionPool::Postgres(_) => DbType::Postgres,
            IndexerConnectionPool::Sqlite(_) => DbType::Sqlite,
        }
    }

    pub async fn connect(database_url: &str) -> Result<IndexerConnectionPool, DatabaseError> {
        let url = url::Url::parse(database_url);
        if url.is_err() {
            return Err(DatabaseError::InvalidConnectionString(database_url.into()));
        }
        let url = url.unwrap();

        match url.scheme() {
            "postgres" => {
                let pool = sqlx::Pool::<sqlx::Postgres>::connect(database_url)
                    .await
                    .expect("Could not connect to postgres backend!");
                Ok(IndexerConnectionPool::Postgres(pool))
            }
            "sqlite" => {
                let pool = sqlx::Pool::<sqlx::Sqlite>::connect(database_url)
                    .await
                    .expect("Could not connect to sqlite backend!");
                Ok(IndexerConnectionPool::Sqlite(pool))
            }
            err => Err(DatabaseError::BackendNotSupported(err.into())),
        }
    }

    pub async fn acquire(&self) -> sqlx::Result<IndexerConnection> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                Ok(IndexerConnection::Postgres(Box::new(p.acquire().await?)))
            }
            IndexerConnectionPool::Sqlite(p) => Ok(IndexerConnection::Sqlite(p.acquire().await?)),
        }
    }
}

#[derive(Debug)]
pub enum IndexerConnection {
    Postgres(Box<PoolConnection<sqlx::Postgres>>),
    Sqlite(PoolConnection<sqlx::Sqlite>),
}

use fuel_indexer_postgres as postgres;
use fuel_indexer_sqlite as sqlite;

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
