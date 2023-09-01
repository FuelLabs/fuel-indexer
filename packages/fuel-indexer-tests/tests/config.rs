use fuel_indexer_lib::config::IndexerConfig;

#[test]
fn test_default_config_file_is_same_as_default_indexer_config() {
    let default_config = IndexerConfig::default();
    let yaml_config = IndexerConfig::from_file("./../../config.yaml").unwrap();
    assert_eq!(
        serde_yaml::to_string(&default_config).unwrap(),
        serde_yaml::to_string(&yaml_config).unwrap()
    );
}
