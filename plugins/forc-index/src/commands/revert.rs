use crate::{ops::forc_index_revert, utils::defaults};
use anyhow::Result;
use clap::Parser;

use std::path::PathBuf;

/// Revert a running indexer to its previous version.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which indexer is deployed.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which indexer is deployed.")]
    pub url: String,

    /// Path of indexer project.
    #[clap(short, long, help = "Path of indexer project.")]
    pub path: Option<PathBuf>,

    /// Path to the manifest of the indexer project being reverted.
    #[clap(
        short,
        long,
        help = "Path to the manifest of the indexer project being reverted."
    )]
    pub manifest: Option<String>,

    /// Authentication header value.
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> Result<(), anyhow::Error> {
    forc_index_revert::init(command).await?;
    Ok(())
}
