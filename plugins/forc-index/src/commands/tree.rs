use crate::ops::forc_index_tree;
use anyhow::Result; 
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Command {
    /// Display info for each file 
    #[clap(short, long, default_value = "true")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> Result<()> {
    forc_index_tree::init(command)?;
    Ok(())
}
