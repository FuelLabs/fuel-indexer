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

    /// Building for native execution.
    #[clap(long, help = "Building for native execution.")]
    pub native: bool,

    /// Directory for all generated artifacts and intermediate files.
    #[clap(
        long,
        help = "Directory for all generated artifacts and intermediate files."
    )]
    pub target_dir: Option<PathBuf>,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            manifest: Some(String::new()),
            path: None,
            debug: false,
            verbose: false,
            locked: false,
            native: false,
            target_dir: Some(std::path::PathBuf::from(".")),
        }
    }
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_build::init(command)?;
    Ok(())
}
