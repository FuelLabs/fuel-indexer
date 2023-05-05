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
    #[error("GraphQL parser error: {0:?}")]
    ParseError(#[from] async_graphql_parser::Error),
    #[error("Could not build schema: {0:?}")]
    SchemaConstructionError(String),
    #[error("Unable to parse join directive: {0:?}")]
    JoinDirectiveError(String),
    #[error("Unable to build schema field and type map: {0:?}")]
    FieldAndTypeConstructionError(String),
}
