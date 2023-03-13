use crate::{ops::forc_index_auth, utils::defaults};
use anyhow::Result;
use clap::Parser;

/// Authenticate against an indexer operator.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to deploy indexer assets
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which to deploy indexer assets.")]
    pub url: String,

    /// Index of account to use for signing.
    #[clap(long, help = "Index of account to use for signing.")]
    pub account: String,

    /// Verbose output.
    #[clap(short, long, help = "Verbose output.")]
    pub verbose: bool,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_auth::init(command)?;
    Ok(())
}
