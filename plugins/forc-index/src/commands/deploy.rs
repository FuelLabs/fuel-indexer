use crate::{ops::forc_index_deploy, utils::defaults};
use anyhow::Result;
use clap::Parser;

/// Deploy an index asset bundle to a remote or locally running indexer server.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to upload index assets
    #[clap(long, default_value = defaults::INDEXER_SERVICE_HOST, help = "URL at which to upload index assets.")]
    pub host: String,

    /// Path of the index manifest to upload
    #[clap(short, long, help = "Path of the index manifest to upload.")]
    pub manifest: String,

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
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_deploy::init(command)?;
    Ok(())
}
