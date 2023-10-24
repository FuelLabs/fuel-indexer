use crate::ops::forc_index_build;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Build an indexer.
#[derive(Debug, Parser)]
pub struct Command {
    /// Manifest file name of indexer being built.
    #[clap(short, long, help = "Manifest file name of indexer being built.")]
    pub manifest: Option<String>,

    /// Path of indexer project.
    #[clap(short, long, help = "Path to the indexer project.")]
    pub path: Option<PathBuf>,

    /// Build optimized artifacts with the debug profile.
    #[clap(short, long, help = "Build artifacts with the debug profile.")]
    pub debug: bool,

    /// Ensure that the Cargo.lock file is up-to-date.
    #[clap(long, help = "Ensure that the Cargo.lock file is up-to-date.")]
    pub locked: bool,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_build::init(command)?;
    Ok(())
}
