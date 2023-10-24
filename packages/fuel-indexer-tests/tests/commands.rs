use duct::cmd;
use fuel_indexer_lib::config::IndexerConfig;

const FUEL_INDEXER: &str = "./../../target/release/fuel-indexer";
const FUEL_INDEXER_API_SERVER: &str = "./../../target/release/fuel-indexer-api-server";
const FORC_INDEX: &str = "./../../target/release/forc-index";

#[test]
fn test_default_indexer_config() {
    let output = serde_yaml::to_string(&IndexerConfig::default()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_help_output() {
    let output = cmd!(FUEL_INDEXER, "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_api_server_help_output() {
    let output = cmd!(FUEL_INDEXER_API_SERVER, "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_run_help_output() {
    let output = cmd!(FUEL_INDEXER, "run", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_fuel_indexer_api_server_run_help_output() {
    let output = cmd!(FUEL_INDEXER_API_SERVER, "run", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_help_output() {
    let output = cmd!(FORC_INDEX, "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_new_help_output() {
    let output = cmd!(FORC_INDEX, "new", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_build_help_output() {
    let output = cmd!(FORC_INDEX, "build", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_deploy_help_output() {
    let output = cmd!(FORC_INDEX, "deploy", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_remove_help_output() {
    let output = cmd!(FORC_INDEX, "remove", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_status_help_output() {
    let output = cmd!(FORC_INDEX, "status", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_auth_help_output() {
    let output = cmd!(FORC_INDEX, "auth", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_kill_help_output() {
    let output = cmd!(FORC_INDEX, "kill", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn test_forc_index_start_help_output() {
    let output = cmd!(FORC_INDEX, "start", "--help")
        .pipe(cmd!("tail", "-n", "+2"))
        .read()
        .unwrap();
    insta::assert_snapshot!(output);
}
