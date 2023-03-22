use crate::{ops::forc_postgres_dropdb, utils::db_dir_or_default};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Drop a database.
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

    /// Remove all database files that might have been persisted to disk.
    #[clap(
        long,
        help = "Remove all database files that might have been persisted to disk."
    )]
    pub remove_persisted: bool,

    /// Verbose output.
    #[clap(short, long, help = "Verbose output.")]
    pub verbose: bool,
}

pub async fn exec(command: Command) -> Result<()> {
    let Command {
        name,
        database_dir,
        config,
        remove_persisted,
        verbose,
    } = command;

    let database_dir = db_dir_or_default(database_dir.as_ref(), &name);

    forc_postgres_dropdb::init(Command {
        name,
        database_dir: Some(database_dir),
        config,
        remove_persisted,
        verbose,
    })
    .await?;

    Ok(())
}
