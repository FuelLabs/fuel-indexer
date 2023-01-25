use crate::ops::forc_index_build;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Build an index.
#[derive(Debug, Parser)]
pub struct Command {
    /// Path of index manifest being built.
    #[clap(short, long, help = "Path of index manifest being built.")]
    pub manifest: Option<PathBuf>,

    /// Path of index project.
    #[clap(short, long, help = "Path to the indexer project.")]
    pub path: Option<PathBuf>,

    /// Target at which to compile.
    #[clap(long, help = "Target at which to compile.")]
    pub target: Option<String>,

    /// Build optimized artifacts with the release profile.
    #[clap(
        short,
        long,
        help = "Build optimized artifacts with the release profile."
    )]
    pub release: bool,

    /// Build with the given profile.
    #[clap(long, help = "Build with the given profile.")]
    pub profile: Option<String>,

    /// Verbose output
    #[clap(short, long, help = "Verbose output.")]
    pub verbose: bool,

    /// Ensure that the Cargo.lock file is up-to-date.
    #[clap(long, help = "Ensure that the Cargo.lock file is up-to-date.")]
    pub locked: bool,

    /// Building for native execution.
    #[clap(long, help = "Building for native execution.")]
    pub native: bool,

    /// Path with which to prefix asset filepaths in the index manifest.
    #[clap(
        long,
        help = "Path with which to prefix asset filepaths in the index manifest."
    )]
    pub output_dir_root: Option<PathBuf>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_build::init(command)?;
    Ok(())
}
