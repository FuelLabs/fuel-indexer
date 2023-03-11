use crate::ops::forc_index_welcome;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Command {
    /// Skip the greeter
    #[clap(long, help = "Skip the greeter.", default_value = "true")]
    pub greeter: bool,
    
    /// Display info for each file
    #[clap(short, long, default_value = "true")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> Result<()> {
    forc_index_welcome::init(command).await?;
    Ok(())
}
