use diesel::result::Error as DieselError;
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

pub use fuel_indexer_lib::config::{
    FuelNodeConfig, GraphQLConfig, IndexerArgs, Parser, PostgresConfig,
};
pub use fuel_indexer_schema::NativeHandlerResult;
pub use fuel_tx::Receipt;
pub use fuel_types::{Address, ContractId};
pub use handler::ReceiptEvent;
pub use manifest::Manifest;
use serde::{Deserialize, Serialize};
pub use service::{IndexerConfig, IndexerService};

pub type IndexerResult<T> = core::result::Result<T, IndexerError>;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Compiler error: {0:#?}")]
    CompileError(#[from] wasmer::CompileError),
    #[error("Error instantiating wasm interpreter: {0:#?}")]
    InstantiationError(#[from] InstantiationError),
    #[error("Error finding exported symbol: {0:#?}")]
    ExportError(#[from] ExportError),
    #[error("Error executing function: {0:#?}")]
    RuntimeError(#[from] RuntimeError),
    #[error("Could not initialize host environment: {0:#?}")]
    HostEnvInitError(#[from] HostEnvInitError),
    #[error("FFI Error {0:?}")]
    FFIError(#[from] ffi::FFIError),
    #[error("Database initialization error: {0:?}")]
    DatabaseInitError(#[from] r2d2::Error),
    #[error("Database query error: {0:?}")]
    DatabaseQueryError(#[from] DieselError),
    #[error("Database connection error: {0:?}")]
    ConnectionError(#[from] diesel::ConnectionError),
    #[error("Missing handler: {0:?}")]
    MissingHandler(String),
    #[error("Indexer transaction error {0:?}")]
    TxError(#[from] crate::executor::TxError),
    #[error("Error executing handler")]
    HandlerError,
    #[error("Unknown error")]
    Unknown,
}

mod handler {
    use crate::{Deserialize, IndexerResult, Serialize};
    use fuel_indexer_schema::NativeHandlerResult;
    use fuel_tx::Receipt;

    pub type Handle = fn(data: Receipt) -> Option<IndexerResult<NativeHandlerResult>>;

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
    pub enum ReceiptEvent {
        // NOTE: Keeping these until https://github.com/FuelLabs/fuel-indexer/pull/65#discussion_r903138005 is figured out
        #[allow(non_camel_case_types)]
        an_event_name,
        #[allow(non_camel_case_types)]
        another_event_name,
        LogData,
        Log,
        ReturnData,
        Other,
    }

    impl From<String> for ReceiptEvent {
        fn from(e: String) -> Self {
            match &e[..] {
                "another_event_name" => ReceiptEvent::another_event_name,
                "an_event_name" => ReceiptEvent::an_event_name,
                "LogData" => ReceiptEvent::LogData,
                "Log" => ReceiptEvent::Log,
                "ReturnData" => ReceiptEvent::ReturnData,
                _ => ReceiptEvent::Other,
            }
        }
    }

    impl From<ReceiptEvent> for String {
        fn from(e: ReceiptEvent) -> String {
            match e {
                ReceiptEvent::another_event_name => "another_event_name".to_owned(),
                ReceiptEvent::an_event_name => "an_event_name".to_owned(),
                ReceiptEvent::LogData => "LogData".to_owned(),
                ReceiptEvent::Log => "Log".to_owned(),
                ReceiptEvent::ReturnData => "ReturnDataa".to_owned(),
                _ => "Other".to_owned(),
            }
        }
    }
}
