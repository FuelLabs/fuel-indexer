use anyhow::Result;
use fuel_indexer_lib::{config::IndexerConfig, defaults};
use pg_embed::{
    pg_enums::PgAuthMethod,
    pg_fetch::{PostgresVersion, PG_V10, PG_V11, PG_V12, PG_V13, PG_V14, PG_V15, PG_V9},
    postgres::PgSettings,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use tracing::info;

pub mod consts {
    pub const PLAIN: &str = "plain";
    pub const MD5: &str = "md5";
    pub const SCRAM_SHA256: &str = "scram-sha-256";

    pub const PG_V15: &str = "v15";
    pub const PG_V14: &str = "v14";
    pub const PG_V13: &str = "v13";
    pub const PG_V12: &str = "v12";
    pub const PG_V11: &str = "v11";
    pub const PG_V10: &str = "v10";
    pub const PG_V9: &str = "v9";
}

pub fn default_indexer_dir() -> PathBuf {
    home::home_dir()
        .expect("Failed to detect home directory.")
        .join(defaults::FUEL_HOME_DIR)
        .join(defaults::INDEXER_CONFIG_DIR)
}

pub fn database_dir_or_default(d: Option<&PathBuf>, name: &str) -> PathBuf {
    d.cloned()
        .unwrap_or_else(|| default_indexer_dir().join(name))
}

pub fn db_config_file_name(name: &str) -> String {
    format!("{name}-db.json")
}

pub fn into_auth_method(s: &str) -> PgAuthMethod {
    match s {
        consts::PLAIN => PgAuthMethod::Plain,
        consts::MD5 => PgAuthMethod::MD5,
        consts::SCRAM_SHA256 => PgAuthMethod::ScramSha256,
        _ => unreachable!(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PgEmbedConfig {
    pub name: String,
    pub user: String,
    pub password: String,
    pub port: u16,
    pub database_dir: Option<PathBuf>,
    pub auth_method: String,
    pub persistent: bool,
    pub timeout: Option<u64>,
    pub migration_dir: Option<PathBuf>,
    pub postgres_version: String,
}

impl From<PgEmbedConfig> for PgSettings {
    fn from(val: PgEmbedConfig) -> Self {
        let PgEmbedConfig {
            database_dir,
            name,
            port,
            user,
            password,
            auth_method,
            persistent,
            timeout,
            migration_dir,
            ..
        } = val;
        Self {
            database_dir: database_dir_or_default(database_dir.as_ref(), &name),
            port,
            user,
            password,
            auth_method: into_auth_method(&auth_method),
            persistent,
            timeout: timeout.map(Duration::from_secs),
            migration_dir,
        }
    }
}

pub fn into_postgres_version(v: &str) -> PostgresVersion {
    match v {
        consts::PG_V15 => PG_V15, // 15.1.0
        consts::PG_V14 => PG_V14, // 14.6.0
        consts::PG_V13 => PG_V13, // 13.9.0
        consts::PG_V12 => PG_V12, // 12.13.0
        consts::PG_V11 => PG_V11, // 11.18.0
        consts::PG_V10 => PG_V10, // 10.23.0
        consts::PG_V9 => PG_V9,   // 9.6.24
        _ => unreachable!(),
    }
}

pub fn load_config_file(name: &str, path: &Path) -> Result<PgEmbedConfig> {
    let filename = db_config_file_name(name);
    let path = path.join(filename);
    let mut file = File::open(path).expect("PgEmbedConfig file does not exist.");
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config: PgEmbedConfig = serde_json::from_str(&content)?;

    Ok(config)
}

pub fn load_pg_config(
    database_dir: Option<&PathBuf>,
    config: Option<&PathBuf>,
    name: &str,
) -> Result<(PathBuf, PgEmbedConfig)> {
    let database_dir = database_dir_or_default(database_dir, name);
    info!("Using database directory at {database_dir:?}");
    fs::create_dir_all(&database_dir)?;
    match config {
        Some(c) => Ok((database_dir, IndexerConfig::from_file(c)?.into())),
        None => {
            let config = load_config_file(name, &database_dir)?;
            Ok((database_dir, config))
        }
    }
}
