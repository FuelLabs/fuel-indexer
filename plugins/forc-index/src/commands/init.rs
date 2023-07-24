use crate::ops::forc_index_init;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Create a new indexer project in the current directory.
#[derive(Debug, Parser)]
pub struct Command {
    /// Name of indexer
    #[clap(long, help = "Name of indexer.")]
    pub name: Option<String>,

    /// Path at which to create indexer.
    #[clap(
        short,
        long,
        parse(from_os_str),
        help = "Path at which to create indexer."
    )]
    pub path: Option<PathBuf>,

    /// Namespace to which indexer belongs.
    #[clap(long, help = "Namespace to which indexer belongs.")]
    pub namespace: Option<String>,

    /// Initialize an indexer with native execution enabled.
    #[clap(long, help = "Initialize an indexer with native execution enabled.")]
    pub native: bool,

    /// Resolve indexer asset filepaths using absolute paths.
    #[clap(long, help = "Resolve indexer asset filepaths using absolute paths.")]
    pub absolute_paths: bool,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_init::init(command)?;
    Ok(())
}
