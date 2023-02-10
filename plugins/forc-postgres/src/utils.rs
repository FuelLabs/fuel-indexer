use fuel_indexer_lib::defaults;
use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    home::home_dir().expect("Failed to detect $HOME directory.")
}

pub fn default_indexer_dir() -> PathBuf {
    home_dir()
        .join(defaults::FUEL_HOME_DIR)
        .join(defaults::INDEXER_CONFIG_DIR)
}

pub fn db_dir_or_default(d: Option<&PathBuf>, name: &str) -> PathBuf {
    d.cloned()
        .unwrap_or_else(|| default_indexer_dir().join(name))
}

pub fn db_config_file_name(name: &str) -> String {
    format!("{name}-db.json")
}
