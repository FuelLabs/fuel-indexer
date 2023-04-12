pub(crate) use crate::commands::run;
use clap::{Parser, Subcommand};
use fuel_indexer_lib::config::IndexerArgs;

#[derive(Parser, Debug)]
#[clap(
    name = "fuel-indexer",
    about = "Fuel Indexer service",
    version,
    rename_all = "kebab-case"
)]

pub struct Opt {
    #[clap(subcommand)]
    command: Indexer,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum Indexer {
    Run(IndexerArgs),
}

pub async fn run_cli() -> anyhow::Result<()> {
    let opt = Opt::try_parse();

    match opt {
        Ok(opt) => match opt.command {
            Indexer::Run(args) => run::exec(args).await,
        },
        Err(e) => e.exit(),
    }
}
