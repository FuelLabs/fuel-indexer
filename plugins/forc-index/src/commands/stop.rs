use crate::{ops::forc_index_stop, utils::defaults};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Stop a running index.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which index is deployed
    #[clap(long, default_value = defaults::DEFAULT_INDEXER_URL, help = "URL at which to upload index assets.")]
    pub url: String,

    /// Path of the index manifest to be parsed
    #[clap(long, help = "Path of the index manifest to be parsed.")]
    pub manifest: PathBuf,

    /// Authentication header value
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_stop::init(command)?;
    Ok(())
}
