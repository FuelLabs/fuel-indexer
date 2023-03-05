use crate::{ops::forc_index_auth, utils::defaults};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Deploy an index asset bundle to a remote or locally running indexer server.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to deploy indexer assets
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which to deploy indexer assets.")]
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

    /// Verbose output.
    #[clap(short, long, help = "Verbose output.")]
    pub verbose: bool,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_auth::init(command)?;
    Ok(())
}
