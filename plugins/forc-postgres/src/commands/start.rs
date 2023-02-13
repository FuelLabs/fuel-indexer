use crate::{utils::db_dir_or_default};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Start PostgreSQL with a database.
#[derive(Debug, Parser)]
pub struct Command {
    /// Name of database.
    #[clap(help = "Name of database.")]
    pub name: String,

    /// Where the PostgreSQL database is stored.
    #[clap(long, help = "Where the PostgreSQL database is stored.")]
    pub database_dir: Option<PathBuf>,

    /// Fuel indexer configuration file.
    #[clap(short, long, help = "Fuel indexer configuration file.")]
    pub config: Option<PathBuf>,
}

pub async fn exec(command: Command) -> Result<()> {
    let Command {
        name,
        database_dir,
        config,
    } = command;

    let database_dir = db_dir_or_default(database_dir.as_ref(), &name);

    Ok(())
}
