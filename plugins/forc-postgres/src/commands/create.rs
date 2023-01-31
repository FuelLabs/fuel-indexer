use crate::ops::forc_postgres_createdb;
use anyhow::Result;
use clap::Parser;
use fuel_indexer_lib::defaults;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Create a new database.
#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Name of database.
    #[clap(help = "Name of database.")]
    pub name: String,

    /// Database user.
    #[clap(short, long, default_value = defaults::POSTGRES_USER, help = "Database user.")]
    pub user: String,

    /// Database password.
    #[clap(short, long, default_value = defaults::POSTGRES_USER, help = "Database password.")]
    pub password: String,

    /// Port to use.
    #[clap(short, long, default_value = defaults::POSTGRES_PORT, help = "Port to use.")]
    pub port: String,

    /// Where to store the PostgreSQL database.
    #[clap(long, help = "Where to store the PostgreSQL database.")]
    pub database_dir: Option<PathBuf>,

    /// Authentication method.
    #[clap(long, default_value = "plain", value_parser(["plain", "md5", "scram-sha-256"]), help = "Authentication method.")]
    pub auth_method: String,

    /// Do not clean up files and directories on databaase drop.
    #[clap(
        long,
        help = "Do not clean up files and directories on databaase drop."
    )]
    pub persistent: bool,

    /// Duration to wait before terminating process execution pg_ctl start/stop and initdb timeout.
    #[clap(
        long,
        help = "Duration to wait before terminating process execution pg_ctl start/stop and initdb timeout."
    )]
    pub timeout: Option<u64>,

    /// The directory containing migrations scripts.
    #[clap(long, help = "The directory containing migrations scripts.")]
    pub migration_dir: Option<PathBuf>,

    /// PostgreSQL version to use.
    #[clap(long, default_value = "v14", value_parser(["v15", "v14", "v13", "v12", "v11", "v10", "v9"]), help = "PostgreSQL version to use.")]
    pub postgres_version: String,

    /// Start the PostgreSQL instance after creation.
    #[clap(long, help = "Start the PostgreSQL instance after creation.")]
    pub start: bool,

    /// Fuel indexer configuration file.
    #[clap(short, long, help = "Fuel indexer configuration file.")]
    pub config: Option<PathBuf>,
}

pub async fn exec(command: Box<Command>) -> Result<()> {
    forc_postgres_createdb::init(*command).await?;
    Ok(())
}
