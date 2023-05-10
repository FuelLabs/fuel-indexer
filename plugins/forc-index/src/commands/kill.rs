use crate::{ops::forc_index_kill, utils::defaults};
use clap::Parser;

/// Kill the indexer process. Note that this command will kill any process
/// listening on the default indexer port or the port specified by the `--port` flag.
#[derive(Debug, Parser)]
pub struct Command {
    /// Port on which the process is listening.
    #[clap(long, default_value = defaults::GRAPHQL_API_PORT, help = "Port at which to detect indexer service API is running.")]
    pub port: String,
}

pub fn exec(command: Command) -> anyhow::Result<()> {
    forc_index_kill::kill(command)?;
    Ok(())
}
