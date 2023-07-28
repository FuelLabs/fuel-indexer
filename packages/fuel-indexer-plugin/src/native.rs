extern crate alloc;
use alloc::vec::Vec;
use async_trait;
use fuel_indexer_schema::FtColumn;

pub use anyhow;
pub use fuel_indexer::prelude::{
    Arc, Database, IndexerError, IndexerResult, IndexerService, Mutex,
    NativeIndexExecutor,
};
pub use fuel_indexer_api_server::api::WebApi;
pub use fuel_indexer_database::{queries, IndexerConnectionPool};
pub use fuel_indexer_lib::{
    config::{cli::Parser, IndexerArgs, IndexerConfig},
    defaults::SERVICE_REQUEST_CHANNEL_SIZE,
    manifest::Manifest,
    utils::{init_logging, ServiceRequest},
};
pub use sha2::{Digest, Sha256};
pub use std::collections::{HashMap, HashSet};
pub use tokio;
pub use tokio::sync::mpsc::channel;
pub use tracing_subscriber;
pub use tracing_subscriber::filter::EnvFilter;

pub struct Logger;

impl Logger {
    pub fn error(log: &str) {
        tracing::error!(log);
    }

    pub fn warn(log: &str) {
        tracing::warn!(log);
    }

    pub fn info(log: &str) {
        tracing::info!(log);
    }

    pub fn debug(log: &str) {
        tracing::debug!(log);
    }

    pub fn trace(log: &str) {
        tracing::trace!(log);
    }
}

#[async_trait::async_trait]
pub trait Entity: Sized + PartialEq + Eq + std::fmt::Debug {
    const TYPE_ID: i64;

    fn from_row(vec: Vec<FtColumn>) -> Self;

    fn to_row(&self) -> Vec<FtColumn>;

    fn type_id(&self) -> i64 {
        Self::TYPE_ID
    }

    async fn load(id: u64) -> Option<Self>;

    async fn save(&self);
}
