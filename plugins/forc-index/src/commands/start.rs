use crate::ops::forc_index_start;
use anyhow::Result;
use clap::Parser;
use fuel_indexer_lib::defaults;
use std::path::PathBuf;

/// Start a local indexer service.
#[derive(Debug, Parser)]
pub struct Command {
    /// Log level passed to the Fuel Indexer service.
    #[clap(long, default_value = "info", value_parser(["info", "debug", "error", "warn"]), help = "Log level passed to the Fuel Indexer service.")]
    pub log_level: String,

    /// Path to the fuel-indexer binary.
    #[clap(long, help = "Path to the fuel-indexer binary.")]
    pub bin: Option<PathBuf>,

    /// Whether to run the Fuel Indexer in the background.
    #[clap(long, help = "Whether to run the Fuel Indexer in the background.")]
    pub background: bool,

    // The following options are taken from fuel_indexer_lib::config::IndexerArgs as
    // we should pass all valid start options to the service
    /// Path to the config file used to start the Fuel Indexer.
    #[clap(long, help = "Path to the config file used to start the Fuel Indexer.")]
    pub config: Option<PathBuf>,

    /// Index config file.
    #[clap(short, long, parse(from_os_str), help = "Index config file.")]
    pub manifest: Option<PathBuf>,

    /// Host of the running Fuel node.
    #[clap(
        long,
        help = "Host of the running Fuel node.",
        default_value = defaults::FUEL_NODE_HOST
    )]
    pub fuel_node_host: String,

    /// Listening port of the running Fuel node.
    #[clap(
        long,
        help = "Listening port of the running Fuel node.",
        default_value = defaults::FUEL_NODE_PORT
    )]
    pub fuel_node_port: String,

    /// GraphQL API host.
    #[clap(long, help = "GraphQL API host.", default_value = defaults::GRAPHQL_API_HOST)]
    pub graphql_api_host: String,

    /// GraphQL API port.
    #[clap(long, help = "GraphQL API port.", default_value = defaults::GRAPHQL_API_PORT)]
    pub graphql_api_port: String,

    /// Database type.
    #[clap(long, help = "Database type.", default_value = defaults::DATABASE, value_parser(["postgres"]))]
    pub database: String,

    /// Postgres username.
    #[clap(long, help = "Postgres username.")]
    pub postgres_user: Option<String>,

    /// Postgres database.
    #[clap(long, help = "Postgres database.")]
    pub postgres_database: Option<String>,

    /// Postgres password.
    #[clap(long, help = "Postgres password.")]
    pub postgres_password: Option<String>,

    /// Postgres host.
    #[clap(long, help = "Postgres host.")]
    pub postgres_host: Option<String>,

    /// Postgres port.
    #[clap(long, help = "Postgres port.")]
    pub postgres_port: Option<String>,

    /// Run database migrations before starting service.
    #[clap(
        long,
        default_value = "true",
        help = "Run database migrations before starting service."
    )]
    pub run_migrations: bool,

    /// Use Prometheus metrics reporting.
    #[clap(
        long,
        default_value = "true",
        help = "Use Prometheus metrics reporting."
    )]
    pub metrics: bool,
}

pub fn exec(command: Box<Command>) -> Result<()> {
    forc_index_start::init(*command)?;
    Ok(())
}
