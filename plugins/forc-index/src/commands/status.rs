use crate::{ops::forc_index_status, utils::defaults};
use clap::Parser;

/// Check the status of a registered indexer.
#[derive(Debug, Parser)]
pub struct Command {
    /// Authentication header value
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,

    /// URL at which to find indexer service.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which to find indexer service.")]
    pub url: String,

    /// Enable verbose logging.
    #[clap(short, long, help = "Enable verbose logging.")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> anyhow::Result<()> {
    forc_index_status::status(command).await
}
