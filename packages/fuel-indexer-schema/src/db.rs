pub mod graphql;
pub mod manager;
pub mod tables;

use fuel_indexer_database::IndexerDatabaseError;
use thiserror::Error;

pub type IndexerSchemaResult<T> = core::result::Result<T, IndexerSchemaError>;

#[derive(Error, Debug)]
pub enum IndexerSchemaError {
    #[error("Error from sqlx: {0:#?}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Database error: {0:?}")]
    DatabaseError(#[from] IndexerDatabaseError),
    #[error("Generic error")]
    Generic,
}
