pub(crate) use crate::commands::run;
use clap::{Parser, Subcommand};
use fuel_indexer_lib::config::ApiServerArgs;

#[derive(Parser, Debug)]
#[clap(
    name = "fuel-indexer-api-server",
    about = "Fuel Indexer API server",
    version,
    rename_all = "kebab-case"
)]

pub struct Opt {
    #[clap(subcommand)]
    command: ApiServer,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum ApiServer {
    Run(ApiServerArgs),
}

pub async fn run_cli() -> anyhow::Result<()> {
    let opt = Opt::try_parse();

    match opt {
        Ok(opt) => match opt.command {
            ApiServer::Run(args) => run::exec(args).await,
        },
        Err(e) => {
            // Prints the error and exits.
            e.exit()
        }
    }
}
