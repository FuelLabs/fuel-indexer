use crate::{defaults, ops::forc_index_deploy};
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

    /// Build optimized artifacts with the release profile.
    #[clap(
        short,
        long,
        help = "Build optimized artifacts with the debug profile."
    )]
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

    /// Enable verbose logging.
    #[clap(short, long, help = "Enable verbose logging.")]
    pub verbose: bool,

    /// Do not build before deploying.
    #[clap(long, help = "Do not build before deploying.")]
    pub skip_build: bool,

    /// Replace an existing indexer with the same UID.
    #[clap(long, help = "If an indexer with the same UID exists, remove it.")]
    pub replace_indexer: bool,

    /// When replacing an indexer, remove all indexed data.
    #[clap(
        long,
        help = "Remove all indexed data when replacing an existing indexer."
    )]
    pub remove_data: bool,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            url: "http://127.0.0.1:29987".to_string(),
            manifest: Some(String::new()),
            path: None,
            auth: Some("".to_string()),
            debug: false,
            verbose: false,
            locked: false,
            native: false,
            skip_build: false,
            replace_indexer: false,
            remove_data: false,
            target_dir: Some(std::path::PathBuf::from(".")),
        }
    }
}
pub async fn exec(command: Command) -> Result<()> {
    forc_index_deploy::init(command).await?;
    Ok(())
}
