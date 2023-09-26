use fuel_indexer_lib::{config::IndexerConfig, utils::Config};
use std::{fs, process::Command};

#[test]
fn test_default_config_file_is_same_as_default_indexer_config() {
    let default_config = IndexerConfig::default();
    let yaml_config = IndexerConfig::from_file("./../../config.yaml").unwrap();
    assert_eq!(
        serde_yaml::to_string(&default_config).unwrap(),
        serde_yaml::to_string(&yaml_config).unwrap()
    );
}

#[test]
fn test_rustc_version_in_default_indexer_cargo_manifest_matches_project_rustc_version() {
    let rustc_version = rustc_version::version().unwrap();
    let _ = Command::new("forc-index")
        .arg("new")
        .arg("indexer-test")
        .arg("--namespace")
        .arg("fuellabs")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    let cargo_toml = fs::read_to_string("./indexer-test/Cargo.toml").unwrap();
    let cargo_toml: Config = toml::from_str(&cargo_toml).unwrap();
    assert_eq!(cargo_toml.package.rust_version, rustc_version.to_string());

    fs::remove_dir_all("./indexer-test").unwrap();
}
