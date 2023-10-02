use crate::ops::forc_index_run_native;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Run a native indexer.
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

    /// Do not build before deploying.
    #[clap(long, help = "Do not build before deploying.")]
    pub skip_build: bool,

    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,

    /// Path to native indexer binary (if not using default location).
    #[clap(
        long,
        help = "Path to native indexer binary (if not using default location)."
    )]
    pub bin: Option<PathBuf>,

    /// Extra passed to `fuel-indexer run`
    ///
    /// Example usage: `forc-index run-native --path . -- --run-migrations --stop-idle-indexers`
    #[clap(
        multiple = true,
        last = true,
        help = "Extra passed to `fuel-indexer run`"
    )]
    pub args: Vec<String>,
}

pub async fn exec(command: Command) -> Result<()> {
    forc_index_run_native::init(command).await?;
    Ok(())
}
