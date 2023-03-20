use forc_index::commands::{
    build::Command as BuildCommand, init::Command as InitCommand,
};
use std::fs::{self, ReadDir};
use std::io;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

#[test]
fn init_command_creates_correct_project_tree() {
    let (_temp_dir, temp_dir_path, temp_dir_name) = init_temp_dir();

    let command = InitCommand {
        name: Some(temp_dir_name.clone()),
        path: Some(temp_dir_path.clone()),
        namespace: temp_dir_name.to_string(),
        native: false,
        absolute_paths: false,
    };

    forc_index::commands::init::exec(command).expect("Init command failed");

    let manifest = temp_dir_name.clone() + ".manifest.yaml";
    let schema_path = format!("schema/{}.schema.graphql", temp_dir_name);

    let expected_project_tree = vec![
        manifest.as_str(),
        "Cargo.toml",
        "src",
        "src/lib.rs",
        "schema",
        schema_path.as_str(),
    ];

    for path in expected_project_tree {
        let full_path = temp_dir_path.join(path);
        assert!(
            full_path.exists(),
            "Expected path '{}' does not exist.",
            full_path.to_string_lossy()
        )
    }
}

//@TODO add negative case for init command

#[test]
fn build_command_creates_artifact_at_expected_path() {
    let (_temp_dir, temp_dir_path, temp_dir_name) = init_temp_dir();

    let command = InitCommand {
        name: Some(temp_dir_name.clone()),
        path: Some(temp_dir_path.clone()),
        namespace: temp_dir_name.to_string(),
        native: false,
        absolute_paths: false,
    };

    forc_index::commands::init::exec(command).expect("Init command failed");

    println!("init success");

    //cd into the test indexer project temp dir
    std::env::set_current_dir(&temp_dir_path).expect("Failed to set current dir");

    //convert temp_dir_name from camelCase to snake-case
    let temp_dir_name = temp_dir_name
        .chars()
        .map(|c| {
            if c.is_uppercase() {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_string()
            }
        })
        .collect::<String>();
    let manifest = temp_dir_name.clone() + ".manifest.yaml";

    let command = BuildCommand {
        target: None,
        native: false,
        path: None,
        verbose: true,
        profile: None,
        release: false,
        locked: false,
        manifest: Some(manifest),
        output_dir_root: None,
    };

    forc_index::commands::build::exec(command).expect("Build command failed");
    let wasm_artifact_path =
        temp_dir_path.join("target/wasm32-unknown-unknown/debug/plugin_test.wasm");

    assert!(
        Path::new(&wasm_artifact_path).exists(),
        "WASM artifact not found at expected path"
    );
}

fn init_temp_dir() -> (TempDir, PathBuf, String) {
    let name = "plugin_test";
    let temp_dir = Builder::new()
        .prefix(name)
        .tempdir()
        .expect("Failed to create temp dir");
    let temp_dir_path = temp_dir.path().to_path_buf();

    println!("temp_dir: {:?}", temp_dir);
    println!("temp_dir_path: {:?}", temp_dir_path);
    println!("temp_dir_name: {:?}", name);

    (temp_dir, temp_dir_path, name.to_string())
}
