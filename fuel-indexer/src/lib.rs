use thiserror::Error;
use wasmer::{ExportError, HostEnvInitError, InstantiationError, RuntimeError};

pub mod api;
mod database;
pub mod executor;
mod ffi;
mod manifest;
mod service;

pub use api::GraphQlApi;
pub use database::{Database, SchemaManager};
pub use executor::{Executor, IndexEnv, NativeIndexExecutor, WasmIndexExecutor};
pub use fuel_indexer_schema::{db::DatabaseError, BlockData, FtColumn};
pub use fuel_types::{Address, ContractId};
pub use manifest::{Manifest, Module};
use serde::{Deserialize, Serialize};
pub use service::{IndexerConfig, IndexerService};

pub type IndexerResult<T> = core::result::Result<T, IndexerError>;

pub const DEFAULT_PORT: u16 = 4000;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Compiler error: {0:#?}")]
    CompileError(#[from] wasmer::CompileError),
    #[error("Error from sqlx: {0:#?}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Error instantiating wasm interpreter: {0:#?}")]
    InstantiationError(#[from] InstantiationError),
    #[error("Error finding exported symbol: {0:#?}")]
    ExportError(#[from] ExportError),
    #[error("Error executing function: {0:#?}")]
    RuntimeError(#[from] RuntimeError),
    #[error("Could not initialize host environment: {0:#?}")]
    HostEnvInitError(#[from] HostEnvInitError),
    #[error("IO Error: {0:#?}")]
    IoError(#[from] std::io::Error),
    #[error("FFI Error {0:?}")]
    FFIError(#[from] ffi::FFIError),
    #[error("Missing handler")]
    MissingHandler,
    #[error("Indexer transaction error {0:?}")]
    TxError(#[from] crate::executor::TxError),
    #[error("Database error {0:?}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Invalid address {0:?}")]
    InvalidAddress(#[from] std::net::AddrParseError),
    #[error("Join Error {0:?}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Error executing handler")]
    HandlerError,
    #[error("No transaction is open!")]
    NoTransactionError,
    #[error("Unknown error")]
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub enum IndexerMessage {
    Blocks(Vec<BlockData>),
    GetObject(u64, u64),
    PutObject(u64, Vec<u8>, Vec<FtColumn>),
    Object(Vec<u8>),
    Commit,
}
