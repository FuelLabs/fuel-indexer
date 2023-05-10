use crate::{ops::forc_postgres_createdb, pg::PostgresVersion, utils::db_dir_or_default};
use anyhow::Result;
use clap::Parser;
use fuel_indexer_lib::defaults;
use pg_embed::postgres::PgEmbed;
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
    #[clap(short, long, default_value = defaults::POSTGRES_PASSWORD, help = "Database password.")]
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

    /// Do not clean up files and directories on database drop.
    #[clap(long, help = "Do not clean up files and directories on database drop.")]
    pub persistent: bool,

    /// Duration to wait before terminating process execution for pg_ctl start/stop and initdb.
    #[clap(
        long,
        help = "Duration to wait before terminating process execution for pg_ctl start/stop and initdb."
    )]
    pub timeout: Option<u64>,

    /// The directory containing migration scripts.
    #[clap(long, help = "The directory containing migration scripts.")]
    pub migration_dir: Option<PathBuf>,

    /// PostgreSQL version to use.
    #[clap(
        long,
        arg_enum,
        default_value = "v14",
        help = "PostgreSQL version to use."
    )]
    pub postgres_version: PostgresVersion,

    /// Start the PostgreSQL instance after creation.
    #[clap(long, help = "Start the PostgreSQL instance after creation.")]
    pub start: bool,

    /// Fuel indexer configuration file.
    #[clap(short, long, help = "Fuel indexer configuration file.")]
    pub config: Option<PathBuf>,

    /// Verbose output.
    #[clap(short, long, help = "Verbose output.")]
    pub verbose: bool,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            name: defaults::POSTGRES_DATABASE.to_string(),
            password: defaults::POSTGRES_PASSWORD.to_string(),
            user: defaults::POSTGRES_USER.to_string(),
            port: defaults::POSTGRES_PORT.to_string(),
            database_dir: None,
            auth_method: "plain".to_string(),
            persistent: true,
            start: false,
            config: None,
            timeout: None,
            migration_dir: None,
            postgres_version: PostgresVersion::V14,
            verbose: false,
        }
    }
}

pub async fn exec(command: Box<Command>) -> Result<PgEmbed> {
    let Command {
        name,
        user,
        password,
        port,
        database_dir,
        auth_method,
        persistent,
        timeout,
        migration_dir,
        postgres_version,
        start,
        config,
        verbose,
    } = *command;

    let database_dir = db_dir_or_default(database_dir.as_ref(), &name);

    let pg = forc_postgres_createdb::init(Command {
        name,
        user,
        password,
        port,
        database_dir: Some(database_dir),
        auth_method,
        persistent,
        timeout,
        migration_dir,
        postgres_version,
        start,
        config,
        verbose,
    })
    .await?;
    Ok(pg)
}
