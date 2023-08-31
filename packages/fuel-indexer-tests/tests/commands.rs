use duct::cmd;
use fuel_indexer_lib::config::IndexerConfig;

#[test]
fn test_default_indexer_config() {
    let output = serde_yaml::to_string(&IndexerConfig::default()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_help_output() {
    let output = cmd!("fuel-indexer", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_api_server_help_output() {
    let output = cmd!("fuel-indexer-api-server", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_run_help_output() {
    let output = cmd!("fuel-indexer", "run", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_api_server_run_help_output() {
    let output = cmd!("fuel-indexer-api-server", "run", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_help_output() {
    let output = cmd!("forc-index", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_new_help_output() {
    let output = cmd!("forc-index", "new", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_build_help_output() {
    let output = cmd!("forc-index", "build", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_deploy_help_output() {
    let output = cmd!("forc-index", "deploy", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_remove_help_output() {
    let output = cmd!("forc-index", "remove", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_status_help_output() {
    let output = cmd!("forc-index", "status", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_auth_help_output() {
    let output = cmd!("forc-index", "auth", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_kill_help_output() {
    let output = cmd!("forc-index", "kill", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_start_help_output() {
    let output = cmd!("forc-index", "start", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}
