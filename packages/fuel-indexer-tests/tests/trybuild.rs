#![allow(unused)]
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone)]
enum TestKind {
    Pass,
    Fail,
}

#[test]
fn test_success_and_failure_macros() {
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

    let tests = vec![
        (
            "fail_if_attribute_manifest_schema_arg_is_invalid.rs",
            "invalid_schema_simple_wasm.yaml",
            TestKind::Fail,
            // Using a custom manifest here
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
            "simple_wasm.yaml",
            TestKind::Pass,
            manifest_content.clone(),
        ),
        (
            "pass_if_indexer_is_valid_multi_type.rs",
            "simple_wasm.yaml",
            TestKind::Pass,
            manifest_content.clone(),
        ),
        (
            "pass_if_unsupported_types_are_used.rs",
            "simple_wasm.yaml",
            TestKind::Pass,
            // Using a custom manifest here
            format!(
                r#"
        namespace: test_namespace
        identifier: simple_wasm_executor
        abi: {tests_root_str}/contracts/simple-wasm/out/debug/contracts-abi-unsupported.json
        graphql_schema: {tests_root_str}/indexers/simple-wasm/schema/simple_wasm.graphql
        contract_id: ~
        module:
            wasm: {project_root_str}/target/wasm32-unknown-unknown/release/simple_wasm.wasm"#
            ),
        ),
        (
            "fail_if_abi_contains_reserved_fuel_type.rs",
            "invalid_abi_type_simple_wasm.yaml",
            TestKind::Fail,
            // Using a custom manifest here
            format!(
                r#"
        namespace: test_namespace
        identifier: simple_wasm_executor
        abi: {tests_root_str}/contracts/simple-wasm/out/debug/contracts-abi-reserved-name.json
        graphql_schema: {tests_root_str}/indexers/simple-wasm/schema/simple_wasm.graphql
        contract_id: ~
        module:
            wasm: {project_root_str}/target/wasm32-unknown-unknown/release/simple_wasm.wasm"#
            ),
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
