use crate::utils::{db_config_file_name, db_dir_or_default};
use anyhow::Result;
use clap::ArgEnum;
use fuel_indexer_lib::config::IndexerConfig;
use pg_embed::{
    pg_enums::PgAuthMethod,
    pg_fetch::{
        PostgresVersion as PgEmbedPostgresVersion, PG_V10, PG_V11, PG_V12, PG_V13,
        PG_V14, PG_V15, PG_V9,
    },
    postgres::PgSettings,
};
use serde::{Deserialize, Serialize};
use std::{fs, fs::File, io::Read, path::PathBuf, time::Duration};
use tracing::info;

pub const PLAIN: &str = "plain";
pub const MD5: &str = "md5";
pub const SCRAM_SHA256: &str = "scram-sha-256";

pub fn into_auth_method(s: &str) -> PgAuthMethod {
    match s {
        PLAIN => PgAuthMethod::Plain,
        MD5 => PgAuthMethod::MD5,
        SCRAM_SHA256 => PgAuthMethod::ScramSha256,
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
    pub postgres_version: PostgresVersion,
}

impl PgEmbedConfig {
    pub fn from_file(
        database_dir: Option<&PathBuf>,
        config: Option<&PathBuf>,
        name: &str,
    ) -> Result<PgEmbedConfig> {
        let database_dir = db_dir_or_default(database_dir, name);
        info!("Using database directory at {database_dir:?}");
        fs::create_dir_all(&database_dir)?;
        match config {
            Some(c) => Ok(IndexerConfig::from_file(c)?.into()),
            None => {
                let filename = db_config_file_name(name);
                let path = database_dir.join(filename);
                let mut file = File::open(&path).expect(&format!(
                    "PgEmbedConfig file {} does not exist.",
                    path.display()
                ));
                let mut content = String::new();
                file.read_to_string(&mut content)?;

                let config: PgEmbedConfig = serde_json::from_str(&content)?;

                Ok(config)
            }
        }
    }
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
            database_dir: db_dir_or_default(database_dir.as_ref(), &name),
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

#[derive(Debug, Clone, ArgEnum, Serialize, Deserialize, Default)]
pub enum PostgresVersion {
    V15,
    #[default]
    V14,
    V13,
    V12,
    V11,
    V10,
    V9,
}

impl PostgresVersion {
    pub fn into_semver(self) -> String {
        match self {
            Self::V15 => "15.1.0".to_string(),
            Self::V14 => "14.6.0".to_string(),
            Self::V13 => "13.9.0".to_string(),
            Self::V12 => "12.13.0".to_string(),
            Self::V11 => "11.18.0".to_string(),
            Self::V10 => "10.23.0".to_string(),
            Self::V9 => "9.6.24".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "v15" => Self::V15,
            "v14" => Self::V14,
            "v13" => Self::V13,
            "v12" => Self::V12,
            "v11" => Self::V11,
            "v10" => Self::V10,
            "v9" => Self::V9,
            _ => unimplemented!(),
        }
    }
}

impl ToString for PostgresVersion {
    fn to_string(&self) -> String {
        match self {
            Self::V15 => "v15".to_string(),
            Self::V14 => "v14".to_string(),
            Self::V13 => "v13".to_string(),
            Self::V12 => "v12".to_string(),
            Self::V11 => "v11".to_string(),
            Self::V10 => "v10".to_string(),
            Self::V9 => "v9".to_string(),
        }
    }
}

impl From<PostgresVersion> for PgEmbedPostgresVersion {
    fn from(val: PostgresVersion) -> Self {
        match val {
            PostgresVersion::V15 => PG_V15,
            PostgresVersion::V14 => PG_V14,
            PostgresVersion::V13 => PG_V13,
            PostgresVersion::V12 => PG_V12,
            PostgresVersion::V11 => PG_V11,
            PostgresVersion::V10 => PG_V10,
            PostgresVersion::V9 => PG_V9,
        }
    }
}
