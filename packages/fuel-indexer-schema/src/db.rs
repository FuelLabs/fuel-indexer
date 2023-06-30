//! # fuel_indexer_schema::db
//!
//! A collection of utilities that contain logic specifically for persisting data to,
//! and fetching data from the database.
//!
//! `fuel_indexer_schema::db` is feature flagged, and not included in indexers by default,
//! due to the fact that this module requires the `sqlx` dependnecy that does not compile
//! to WebAssembly (WASM).

pub mod manager;
pub mod tables;

use fuel_indexer_database::IndexerDatabaseError;
use thiserror::Error;

/// Result type returned by indexer schema database-specfici operations.
pub type IndexerSchemaDbResult<T> = core::result::Result<T, IndexerSchemaDbError>;

/// Error type returned by indexer schema database-specfici operations.
#[derive(Error, Debug)]
pub enum IndexerSchemaDbError {
    #[error("Error from sqlx: {0:#?}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Database error: {0:?}")]
    DatabaseError(#[from] IndexerDatabaseError),
    #[error("Generic error")]
    Generic,
    #[error("Could not build schema: {0:?}")]
    SchemaConstructionError(String),
    #[error("Unable to parse join directive: {0:?}")]
    JoinDirectiveError(String),
    #[error("Unable to build schema field and type map: {0:?}")]
    FieldAndTypeConstructionError(String),
    #[error("This TypeKind is unsupported.")]
    UnsupportedTypeKind,
    #[error("List types are unsupported.")]
    ListTypesUnsupported,
    #[error("IndexerSchemaError: {0:?}")]
    IndexerSchemaError(#[from] crate::IndexerSchemaError),
    #[error("Utf8 Error: {0:?}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Unable to parse GraphQL schema: {0:?}")]
    ParsedError(#[from] fuel_indexer_lib::graphql::ParsedError),
    #[error("Manifest Error: {0:?}")]
    ManifestError(#[from] fuel_indexer_lib::manifest::ManifestError),
}
