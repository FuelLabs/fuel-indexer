use crate::{ops::forc_index_revert, utils::defaults};
use anyhow::Result;
use clap::Parser;

use std::path::PathBuf;

/// Revert the running index to the previous version.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which index is deployed.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which index is deployed.")]
    pub url: String,

    /// Path of index project.
    #[clap(short, long, help = "Path of index project.")]
    pub path: Option<PathBuf>,

    /// Path to the manifest of indexer project being deployed.
    #[clap(
        short,
        long,
        help = "Path to the manifest of indexer project being deployed."
    )]
    pub manifest: Option<String>,

    /// Authentication header value.
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.", default_value = "true")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> Result<(), anyhow::Error> {
    forc_index_revert::init(command).await?;
    Ok(())
}
