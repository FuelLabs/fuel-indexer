use crate::{ops::forc_index_deploy, utils::defaults};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Deploy an index asset bundle to a remote or locally running indexer server.
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

    /// Path of index project.
    #[clap(short, long, help = "Path to the indexer project.")]
    pub path: Option<PathBuf>,

    /// Authentication header value
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,

    // The following args are passed to `forc index build`
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

impl Default for Command {
    fn default() -> Self {
        Command {
            url: "http://127.0.0.1:29987".to_string(),
            manifest: Some(String::new()),
            path: None,
            auth: Some("".to_string()),
            target: Some(defaults::WASM_TARGET.to_string()),
            release: true,
            profile: Some("release".to_string()),
            verbose: false,
            locked: false,
            native: false,
            output_dir_root: Some(std::path::PathBuf::from(".")),
        }
    }
}
pub fn exec(command: Command) -> Result<()> {
    forc_index_deploy::init(command)?;
    Ok(())
}
