use crate::{ops::forc_index_revert, utils::defaults};
use anyhow::Result;
use clap::Parser;
use fuel_indexer_lib::config::IndexerArgs;
use std::path::PathBuf;
/// Revert the running index to the previous version.

#[derive(Debug, Parser)]
pub struct Command{
    /// URL at which index is deployed.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which index is deployed.")]
    pub url: String,

    /// Path to the manifest of the indexer project being reverted. 
    #[clap(
        short,
        long,
        help = "Path to the manifest of indexer project being deployed."
    )]
    pub manifest: Option<String>,

    /// Path of index project.
    #[clap(short, long, help = "Path to the indexer project.")]
    pub path: Option<PathBuf>,

    /// Authentication header value.
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_revert::init(command)?;
    Ok(())
}
