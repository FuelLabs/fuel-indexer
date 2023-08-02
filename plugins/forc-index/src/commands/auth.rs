use crate::{defaults, ops::forc_index_auth};
use anyhow::Result;
use clap::Parser;
use fuel_indexer_lib::defaults::ACCOUNT_INDEX;

/// Authenticate against an indexer service.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to deploy indexer assets.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which to deploy indexer assets.")]
    pub url: String,

    /// Index of account to use for signing.
    #[clap(long, default_value = ACCOUNT_INDEX, help = "Index of account to use for signing.")]
    pub account: String,

    /// Verbose output.
    #[clap(short, long, help = "Verbose output.")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> Result<()> {
    forc_index_auth::init(command).await?;
    Ok(())
}
