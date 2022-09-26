pub use fuel_indexer_database_types::*;
use fuel_indexer_lib::utils::ServiceStatus;
use fuel_indexer_postgres as postgres;
use fuel_indexer_sqlite as sqlite;
use sqlx::{pool::PoolConnection, Error as SqlxError};
use std::cmp::Ordering;
use thiserror::Error;
use tokio::time::{sleep, Duration};
use tracing::warn;

pub mod queries;

const DB_CONN_ATTEMPTS: usize = 5;
const DB_CONN_RETRY_FACTOR: u64 = 2;

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

        attempt_connection(database_url, url.scheme()).await
    }

    pub async fn is_connected(&self) -> sqlx::Result<ServiceStatus> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                let mut conn = p.acquire().await.expect("Failed to get pool connection");
                let result = postgres::execute_query(&mut conn, "SELECT true;".to_string())
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
            IndexerConnectionPool::Sqlite(p) => Ok(IndexerConnection::Sqlite(p.acquire().await?)),
        }
    }
}

async fn attempt_connection(
    database_url: &str,
    scheme: &str,
) -> Result<IndexerConnectionPool, DatabaseError> {
    let mut remaining_retries = DB_CONN_ATTEMPTS;
    let mut delay = DB_CONN_RETRY_FACTOR;

    match scheme {
        "postgres" => {
            let pool = loop {
                match sqlx::Pool::<sqlx::Postgres>::connect(database_url).await {
                    Ok(p) => break p,
                    Err(_) => {
                        if remaining_retries > 0 {
                            warn!(
                                "Could not connect to postgres backend, retrying in {} seconds...",
                                delay
                            );
                            remaining_retries -= 1;
                            sleep(Duration::from_secs(delay)).await;
                            delay *= 2;
                        } else {
                            panic!(
                                "Retry attempts exceeded; could not connect to postgres backend!"
                            )
                        }
                    }
                }
            };

            Ok(IndexerConnectionPool::Postgres(pool))
        }
        "sqlite" => {
            let pool = loop {
                match sqlx::Pool::<sqlx::Sqlite>::connect(database_url).await {
                    Ok(p) => break p,
                    Err(_) => {
                        if remaining_retries > 0 {
                            warn!(
                                "Could not connect to sqlite backend, retrying in {} seconds...",
                                delay
                            );
                            remaining_retries -= 1;
                            sleep(Duration::from_secs(delay)).await;
                            delay *= 2;
                        } else {
                            panic!("Retry attempts exceeded; could not connect to sqlite backend!")
                        }
                    }
                }
            };

            Ok(IndexerConnectionPool::Sqlite(pool))
        }
        err => Err(DatabaseError::BackendNotSupported(err.into())),
    }
}
