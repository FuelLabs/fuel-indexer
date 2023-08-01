use crate::ops::forc_index_check;
use anyhow::Result;
use clap::Parser;

// Command has to be created in order for the help
// message to be shown in the primary help output
/// Check for Fuel indexer components
#[derive(Debug, Parser)]
pub struct Command {}

pub async fn exec() -> Result<()> {
    forc_index_check::init().await
}
