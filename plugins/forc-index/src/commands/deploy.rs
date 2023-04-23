use crate::{ops::forc_index_deploy, utils::defaults};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Deploy an indexer to an indexer service.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to deploy indexer assets
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which to deploy indexer assets.")]
    pub url: String,

    /// Path to the manifest of indexer project being deployed.
    #[clap(
        short,
        long,
        help = "Path to the manifest of indexer project being deployed."
    )]
    pub manifest: Option<String>,

    /// Path of indexer project.
    #[clap(short, long, help = "Path to the indexer project.")]
    pub path: Option<PathBuf>,

    /// Authentication header value
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,

    /// Target at which to compile.
    #[clap(long, default_value = defaults::INDEXER_TARGET, help = "Target at which to compile.")]
    pub target: String,

    /// Build optimized artifacts with the release profile.
    #[clap(
        short,
        long,
        help = "Build optimized artifacts with the release profile.",
        default_value = defaults::BUILD_RELEASE_PROFILE,
    )]
    pub release: String,

    /// Build with the given profile.
    #[clap(long, help = "Build with the given profile.")]
    pub profile: Option<String>,

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

    /// Enable verbose logging.
    #[clap(short, long, help = "Enable verbose logging.")]
    pub verbose: bool,

    /// Do not build before deploying.
    #[clap(long, help = "Do not build before deploying.")]
    pub skip_build: bool,

    /// Stop the previous running indexer.
    #[clap(long, help = "Stop previous running indexer.")]
    pub stop_previous: bool,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            url: "http://127.0.0.1:29987".to_string(),
            manifest: Some(String::new()),
            path: None,
            auth: Some("".to_string()),
            target: defaults::WASM_TARGET.to_string(),
            release: true.to_string(),
            profile: Some("release".to_string()),
            verbose: false,
            locked: false,
            native: false,
            skip_build: false,
            target_dir: Some(std::path::PathBuf::from(".")),
            stop_previous: true,
        }
    }
}
pub fn exec(command: Command) -> Result<()> {
    forc_index_deploy::init(command)?;
    Ok(())
}
