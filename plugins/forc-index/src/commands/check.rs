use crate::{ops::forc_index_check, utils::defaults};
use anyhow::Result;
use clap::Parser;

/// Check to ensure that all indexer components are up and running.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to upload index assets
    #[clap(long, default_value = defaults::DEFAULT_INDEXER_URL, help = "URL at which to find indexer service.")]
    pub url: String,

    #[clap(long, default_value = defaults::DEFAULT_GRAPHQ_API_PORT)]
    pub grpahql_api_port: String,

    #[clap(long)]
    pub database_uri: Option<String>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_check::init(command)?;
    Ok(())
}
