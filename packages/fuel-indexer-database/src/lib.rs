use fuel_indexer_lib::utils::{attempt_database_connection, ServiceStatus};
use fuel_indexer_postgres as postgres;
use fuel_indexer_sqlite as sqlite;
use sqlx::{
    pool::PoolConnection, postgres::PgConnectOptions, sqlite::SqliteConnectOptions,
    Error as SqlxError,
};
use std::{cmp::Ordering, str::FromStr};
use thiserror::Error;

pub mod queries;

#[derive(Debug, Error)]
pub enum IndexerDatabaseError {
    #[error("Invalid connection string: {0:?}")]
    InvalidConnectionString(String),
    #[error("Database backend not supported: {0:?}")]
    BackendNotSupported(String),
    #[error("No transaction is open.")]
    NoTransactionError,
    #[error("Error from sqlx: {0:#?}")]
    SqlxError(#[from] SqlxError),
    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug)]
pub enum IndexerConnection {
    Postgres(Box<PoolConnection<sqlx::Postgres>>),
    Sqlite(PoolConnection<sqlx::Sqlite>),
}

#[derive(Clone, Debug)]
pub enum IndexerConnectionPool {
    Postgres(sqlx::Pool<sqlx::Postgres>),
    Sqlite(sqlx::Pool<sqlx::Sqlite>),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum DbType {
    Postgres,
    Sqlite,
}

impl Default for DbType {
    fn default() -> DbType {
        DbType::Postgres
    }
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

    pub async fn connect(
        database_url: &str,
    ) -> Result<IndexerConnectionPool, IndexerDatabaseError> {
        let url = url::Url::parse(database_url);
        if url.is_err() {
            return Err(IndexerDatabaseError::InvalidConnectionString(
                database_url.into(),
            ));
        }
        let url = url.expect("Database URL should be correctly formed");
        match url.scheme() {
            "postgres" => {
                let pool = attempt_database_connection(|| {
                    let connection_options = PgConnectOptions::from_str(database_url)
                        .expect("Failed to parse Postgres connection options");
                    sqlx::postgres::PgPoolOptions::new().connect_with(connection_options)
                })
                .await;

                Ok(IndexerConnectionPool::Postgres(pool))
            }
            "sqlite" => {
                let pool = attempt_database_connection(|| {
                    let connection_options = SqliteConnectOptions::from_str(database_url)
                        .expect("Failed to parse SQLite connection options");
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .connect_with(connection_options)
                })
                .await;

                Ok(IndexerConnectionPool::Sqlite(pool))
            }
            err => Err(IndexerDatabaseError::BackendNotSupported(err.into())),
        }
    }

    pub async fn is_connected(&self) -> sqlx::Result<ServiceStatus> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                let mut conn = p.acquire().await.expect("Failed to get pool connection");
                let result =
                    postgres::execute_query(&mut conn, "SELECT true;".to_string())
                        .await
                        .expect("Failed to test Postgres connection.");

                match result.cmp(&1) {
                    Ordering::Equal => Ok(ServiceStatus::OK),
                    _ => Ok(ServiceStatus::NotOk),
                }
            }
            IndexerConnectionPool::Sqlite(p) => {
                let mut conn = p.acquire().await.expect("Failed to get pool connection");
                let result = sqlite::execute_query(&mut conn, "SELECT true;".to_string())
                    .await
                    .expect("Failed to test Sqlite connection.");

                match result.cmp(&1) {
                    Ordering::Equal => Ok(ServiceStatus::OK),
                    _ => Ok(ServiceStatus::NotOk),
                }
            }
        }
    }

    pub async fn acquire(&self) -> sqlx::Result<IndexerConnection> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                Ok(IndexerConnection::Postgres(Box::new(p.acquire().await?)))
            }
            IndexerConnectionPool::Sqlite(p) => {
                Ok(IndexerConnection::Sqlite(p.acquire().await?))
            }
        }
    }
}
