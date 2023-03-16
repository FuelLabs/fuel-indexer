use crate::{ops::forc_index_remove, utils::defaults};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Stop and remove a running index.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which index is deployed.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which index is deployed.")]
    pub url: String,

    /// Path to the manifest of indexer project being deployed.
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

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.", default_value = "true")]
    pub verbose: bool,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_remove::init(command)?;
    Ok(())
}
