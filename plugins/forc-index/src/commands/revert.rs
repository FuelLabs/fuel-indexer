use crate::{ops::forc_index_revert};
use anyhow::Result;
use fuel_indexer_lib::config::IndexerArgs;

use std::path::PathBuf;
/// Revert the running index to the previous version.

pub struct Command{
    pub manifest: Option<String>,

    /// Path of index project.
    pub path: Option<PathBuf>,

    /// Authentication header value.
    pub auth: Option<String>,

    /// URL of the indexer.
    pub url: String,

    /// Indexer start 
    pub indexer_start: Option<IndexerArgs>
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_revert::init(command)?;
    Ok(())
}
