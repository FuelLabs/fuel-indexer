extern crate alloc;
use alloc::vec::Vec;
pub use anyhow;
pub use fuel_indexer::prelude::{IndexerError, IndexerResult, IndexerService};

#[cfg(feature = "native-execution")]
pub use fuel_indexer::prelude::NativeIndexExecutor;

pub use fuel_indexer_api_server::api::GraphQlApi;
pub use fuel_indexer_database::{queries, IndexerConnectionPool};
pub use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig, Parser},
    defaults::SERVICE_REQUEST_CHANNEL_SIZE,
    manifest::Manifest,
    utils::ServiceRequest,
};
use fuel_indexer_schema::FtColumn;
pub use futures;
pub use tokio;
pub use tokio::sync::mpsc::channel;
pub use tracing::{debug, error, info, trace, warn};
pub use tracing_subscriber;
pub use tracing_subscriber::filter::EnvFilter;

pub mod types {
    pub use fuel_indexer_schema::FtColumn;
    pub use fuel_indexer_types::{abi as fuel, tx, *};
}

pub mod utils {
    pub use fuel_indexer_lib::utils::sha256_digest;
}

pub struct Logger;

impl Logger {
    pub fn error(log: &str) {
        error!(log);
    }

    pub fn warn(log: &str) {
        warn!(log);
    }

    pub fn info(log: &str) {
        info!(log);
    }

    pub fn debug(log: &str) {
        debug!(log);
    }

    pub fn trace(log: &str) {
        trace!(log);
    }
}

pub trait Entity: Sized + PartialEq + Eq + std::fmt::Debug {
    const TYPE_ID: u64;

    fn from_row(vec: Vec<FtColumn>) -> Self;

    fn to_row(&self) -> Vec<FtColumn>;

    fn type_id(&self) -> u64 {
        Self::TYPE_ID
    }

    fn load(_id: u64) -> Option<Self> {
        None
    }

    fn save(&self) {}
}
