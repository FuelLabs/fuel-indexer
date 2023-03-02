use crate::ops::forc_index_welcome;
use clap::Parser;
use anyhow::Result;

#[derive(Debug, Parser)]
pub struct Command {
    /// The URL of the Fuel Indexer API
    #[clap(short, long, default_value = "http://localhost:8080")]
    pub url: String,
    /// The path to the project directory
    #[clap(short, long, default_value = ".")]
    pub path: String,
    /// The path to the manifest file
    #[clap(short, long, default_value = "manifest.yaml")]
    pub manifest: String,
    /// The authorization token
    #[clap(short, long)]
    pub auth: Option<String>,
}

pub async fn exec(command: Command) -> Result<()> {
    forc_index_welcome::init(command).await?;
    Ok(())
}
