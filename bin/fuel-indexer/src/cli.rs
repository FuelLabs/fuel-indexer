use clap::{Parser, Subcommand};
use fuel_indexer_lib::config::IndexerArgs;
use std::{env, str::FromStr};
use tracing_subscriber::filter::EnvFilter;

pub mod run;

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

pub const LOG_FILTER: &str = "RUST_LOG";
pub const HUMAN_LOGGING: &str = "HUMAN_LOGGING";

pub async fn init_logging() -> anyhow::Result<()> {
    let filter = match env::var_os(LOG_FILTER) {
        Some(_) => {
            EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
        }
        None => EnvFilter::new("info"),
    };

    let human_logging = env::var_os(HUMAN_LOGGING)
        .map(|s| {
            bool::from_str(s.to_str().unwrap())
                .expect("Expected `true` or `false` to be provided for `HUMAN_LOGGING`")
        })
        .unwrap_or(true);

    let sub = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter);

    if human_logging {
        sub.with_ansi(true)
            .with_level(true)
            .with_line_number(true)
            .init();
    } else {
        sub.with_ansi(false)
            .with_level(true)
            .with_line_number(true)
            .json()
            .init();
    }
    Ok(())
}

pub async fn run_cli() -> anyhow::Result<()> {
    init_logging().await?;

    let opt = Opt::try_parse();

    match opt {
        Ok(opt) => match opt.command {
            Indexer::Run(args) => run::exec(args).await,
        },
        Err(e) => {
            // Prints the error and exits.
            e.exit()
        }
    }
}
