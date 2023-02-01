pub use fuel_indexer_database_types::DbType;
use fuel_indexer_lib::utils::{attempt_database_connection, ServiceStatus};
use fuel_indexer_postgres as postgres;
use sqlx::{pool::PoolConnection, postgres::PgConnectOptions, Error as SqlxError};
use std::{cmp::Ordering, str::FromStr};
use thiserror::Error;

pub mod queries;
pub mod types {
    pub use fuel_indexer_database_types::*;
}

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
}

#[derive(Clone, Debug)]
pub enum IndexerConnectionPool {
    Postgres(sqlx::Pool<sqlx::Postgres>),
}

impl IndexerConnectionPool {
    pub fn database_type(&self) -> DbType {
        match self {
            IndexerConnectionPool::Postgres(_) => DbType::Postgres,
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
                    sqlx::postgres::PgPoolOptions::new().connect_with(
                        PgConnectOptions::from_str(database_url).unwrap_or_else(|e| {
                            panic!("Could not derive PgConnectOptions: {e}",)
                        }),
                    )
                })
                .await;

                Ok(IndexerConnectionPool::Postgres(pool))
            }
            err => Err(IndexerDatabaseError::BackendNotSupported(err.into())),
        }
    }

    pub async fn is_connected(&self) -> sqlx::Result<ServiceStatus> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                let mut conn = p.acquire().await?;
                let result =
                    postgres::execute_query(&mut conn, "SELECT true;".to_string())
                        .await?;

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
        }
    }
}
