#![allow(unused)]
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone)]
enum TestKind {
    Pass,
    Fail,
}

fn test_dirs() -> (String, String, String) {
    let t = trybuild::TestCases::new();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::env::set_var("COMPILE_TEST_PREFIX", manifest_dir);

    let project_root =
        std::fs::canonicalize(std::path::Path::new(manifest_dir).join("..").join(".."))
            .unwrap();
    let project_root_str = project_root.to_str().unwrap();
    let tests_root = project_root.join("packages").join("fuel-indexer-tests");
    let tests_root_str = tests_root.to_str().unwrap();
    let trybuild_root = tests_root.join("trybuild");
    let abi_root = trybuild_root.join("abi");
    let abi_root_str = abi_root.to_str().unwrap();

    (
        abi_root_str.to_string(),
        tests_root_str.to_string(),
        project_root_str.to_string(),
    )
}

fn manifest_with_contract_abi(contract_name: &str) -> String {
    let (abi_root_str, tests_root_str, project_root_str) = test_dirs();
    format!(
        r#"
namespace: test_namespace
identifier: simple_wasm_executor
abi: {abi_root_str}/{contract_name}
graphql_schema: {tests_root_str}/indexers/simple-wasm/schema/simple_wasm.graphql
contract_id: ~
module:
    wasm: {project_root_str}/target/wasm32-unknown-unknown/release/simple_wasm.wasm"#
    )
}

#[test]
fn test_success_and_failure_macros() {
    let (abi_root_str, tests_root_str, project_root_str) = test_dirs();
    let manifest_content = format!(
        r#"
namespace: test_namespace
identifier: simple_wasm_executor
abi: {tests_root_str}/contracts/simple-wasm/out/debug/contracts-abi.json
graphql_schema: {tests_root_str}/indexers/simple-wasm/schema/simple_wasm.graphql
contract_id: ~
module:
  wasm: {project_root_str}/target/wasm32-unknown-unknown/release/simple_wasm.wasm
"#
    );

    // IMPORTANT: Even though in theory we should be able to just re-use the same filename
    // since we're writing and reading to each file for each test individually, in practice,
    // these tests will error out if we use the same filename for each test.
    //
    // So, we simply change the manifest name according to each test to avoid these flaky errors.
    let tests = vec![
        (
            "fail_if_attribute_manifest_schema_arg_is_invalid.rs",
            "invalid_schema_simple_wasm.yaml",
            TestKind::Fail,
            format!(
                r#"
        namespace: test_namespace
        identifier: simple_wasm_executor
        abi: {tests_root_str}/contracts/simple-wasm/out/debug/contracts-abi.json
        # This schema file doesn't actually exist
        graphql_schema: schema.graphql
        contract_id: ~
        module:
            wasm: {project_root_str}/target/wasm32-unknown-unknown/release/simple_wasm.wasm"#
            ),
        ),
        (
            "fail_if_attribute_args_include_self.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "fail_if_attribute_args_not_included.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "fail_if_attribute_abi_arg_includes_invalid_type.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "fail_if_indexer_module_is_empty.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "fail_if_arg_not_passed_to_handler_function.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "pass_if_indexer_is_valid_single_type.rs",
            "simple_wasm_single.yaml",
            TestKind::Pass,
            manifest_content.clone(),
        ),
        (
            "pass_if_indexer_is_valid_multi_type.rs",
            "simple_wasm_multi.yaml",
            TestKind::Pass,
            manifest_content.clone(),
        ),
        (
            "pass_if_unsupported_types_are_used.rs",
            "simple_wasm_unsupported.yaml",
            TestKind::Pass,
            manifest_content.clone(),
        ),
        (
            "fail_if_abi_contains_reserved_fuel_type.rs",
            "invalid_abi_type_simple_wasm.yaml",
            TestKind::Fail,
            manifest_with_contract_abi("contracts-abi-reserved-name.json"),
        ),
        (
            "fail_if_ident_not_defined_in_abi.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "fail_if_non_function_patterns_included_in_module.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "fail_if_unsupported_type_used_in_handler_args.rs",
            "simple_wasm.yaml",
            TestKind::Fail,
            manifest_content.clone(),
        ),
        (
            "pass_if_using_sway_amm_abi.rs",
            "sway_amm.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("AMM-contract-abi.json"),
        ),
        (
            "pass_if_using_sway_dao_abi.rs",
            "sway_dao.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("DAO-contract-abi.json"),
        ),
        (
            "pass_if_using_sway_asset_contract_abi.rs",
            "asset_contract.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("asset-contract-abi.json"),
        ),
        (
            "pass_if_using_sway_distributor_contract_abi.rs",
            "distributor_contract.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("distributor-contract-abi.json"),
        ),
        (
            "pass_if_using_sway_escrow_contract_abi.rs",
            "escrow_contract.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("escrow-contract-abi.json"),
        ),
        (
            "pass_is_using_sway_exchange_contract_abi.rs",
            "exchange_contract.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("exchange-contract-abi.json"),
        ),
        // NOTE: I don't think the ABI tokens are being properly generated from this contract JSON
        // (
        //     "pass_if_using_sway_multisig_contract_abi.rs",
        //     "multisig_contract.yaml",
        //     TestKind::Pass,
        //     manifest_with_contract_abi("multisig-contract-abi.json"),
        // ),
        (
            "pass_if_using_sway_oracle_contract_abi.rs",
            "oracle_contract.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("oracle-contract-abi.json"),
        ),
        (
            "pass_if_using_sway_registry_contract_abi.rs",
            "registry_contract.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("registry-contract-abi.json"),
        ),
        (
            "pass_if_using_swap_predicate_abi.rs",
            "predicate_abi.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("swap-predicate-abi.json"),
        ),
        (
            "pass_if_using_timelock_contract_abi.rs",
            "timelock_contract.yaml",
            TestKind::Pass,
            manifest_with_contract_abi("timelock-contract-abi.json"),
        ),
    ];

    for (name, manifest_name, kind, manifest_content) in tests {
        let manifest_path = trybuild_root.join(manifest_name);
        let mut f = std::fs::File::create(&manifest_path).unwrap();
        f.write_all(manifest_content.as_bytes()).unwrap();

        match kind {
            TestKind::Pass => {
                t.pass(trybuild_root.join(&name));
            }
            TestKind::Fail => {
                t.compile_fail(trybuild_root.join(&name));
            }
        }
    }
}
