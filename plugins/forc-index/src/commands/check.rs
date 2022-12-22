use crate::{ops::forc_index_check, utils::defaults};
use anyhow::Result;
use clap::Parser;

/// Get status checks on all indexer components.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to find indexer service.
    #[clap(long, default_value = defaults::INDEXER_SERVICE_URL, help = "URL at which to find indexer service.")]
    pub url: String,

    /// Port at which to detect indexer service API is running.
    #[clap(long, default_value = defaults::GRAPHQL_API_PORT, help = "Port at which to detect indexer service API is running.")]
    pub grpahql_api_port: String,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_check::init(command)?;
    Ok(())
}
