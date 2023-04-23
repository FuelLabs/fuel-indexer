use crate::ops::forc_index_pull_abi;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Command {
    ////// URL of the ABI file
    #[clap(long, help = "URL of the ABI file.")]
    pub raw_url: String,

    /// Name of contract
    #[clap(long, help = "Name of contract.")]
    pub contract_name: Option<String>,

    /// Path at which to write the ABI.
    #[clap(
        short,
        long,
        parse(from_os_str),
        help = "Path at which to write the ABI."
    )]
    pub path: Option<PathBuf>,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> Result<(), anyhow::Error> {
    forc_index_pull_abi::init(command).await?;
    Ok(())
}
