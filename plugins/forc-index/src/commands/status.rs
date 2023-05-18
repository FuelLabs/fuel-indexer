use crate::{ops::forc_index_status, utils::defaults};
use clap::Parser;

/// Get a basic status check of registered indexers.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to find indexer service.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which to find indexer service.")]
    pub url: String,
}

pub async fn exec(command: Command) -> anyhow::Result<()> {
    forc_index_status::status(command).await
}
