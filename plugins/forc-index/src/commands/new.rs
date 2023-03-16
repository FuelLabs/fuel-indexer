use crate::ops::forc_index_new;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Create a new indexer project in a new directory.
#[derive(Debug, Parser)]
pub struct Command {
    /// Name of index
    #[clap(long, help = "Name of index.")]
    pub name: Option<String>,

    /// Path at which to create index
    pub path: PathBuf,

    /// Name of the index namespace
    #[clap(long, help = "Namespace in which index belongs.")]
    pub namespace: String,

    /// Whether to initialize an index with native execution enabled
    #[clap(
        long,
        help = "Whether to initialize an index with native execution enabled."
    )]
    pub native: bool,

    /// Resolve index asset filepaths using absolute paths.
    #[clap(long, help = "Resolve index asset filepaths using absolute paths.")]
    pub absolute_paths: bool,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.", default_value = "true")]
    pub verbose: bool,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_new::init(command)?;
    Ok(())
}
