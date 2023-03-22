use crate::{ops::forc_index_remove, utils::defaults};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Stop and remove a running indexer.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which indexer is deployed.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which indexer is deployed.")]
    pub url: String,

    /// Path to the manifest of the indexer project being removed.
    #[clap(
        short,
        long,
        help = "Path to the manifest of the indexer project being removed."
    )]
    pub manifest: Option<String>,

    /// Path of indexer project.
    #[clap(short, long, help = "Path to the indexer project.")]
    pub path: Option<PathBuf>,

    /// Authentication header value.
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_remove::init(command)?;
    Ok(())
}
