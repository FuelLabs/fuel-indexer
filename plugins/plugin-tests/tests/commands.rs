use forc_index::commands::{
    build::Command as BuildCommand, init::Command as InitCommand,
    new::Command as NewCommand,
};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

#[test]
fn init_command_creates_correct_project_tree() {
    let (_temp_dir, temp_dir_path, temp_dir_name) = init_temp_dir();

    forc_index::commands::init::exec(InitCommand {
        name: Some(temp_dir_name.clone()),
        path: Some(temp_dir_path.clone()),
        namespace: temp_dir_name.to_string(),
        native: false,
        absolute_paths: false,
    })
    .expect("Init command failed");

    let manifest = temp_dir_name.clone() + ".manifest.yaml";
    let schema_path = format!("schema/{}.schema.graphql", temp_dir_name);

    [
        manifest.as_str(),
        "Cargo.toml",
        "src",
        "src/lib.rs",
        "schema",
        schema_path.as_str(),
    ]
    .iter()
    .map(|path| temp_dir_path.join(path))
    .for_each(|full_path| {
        assert!(
            full_path.exists(),
            "Expected path '{}' does not exist.",
            full_path.to_string_lossy()
        )
    });
}

#[test]
fn build_command_creates_artifact_at_expected_path() {
    let (_temp_dir, temp_dir_path, temp_dir_name) = init_temp_dir();

    forc_index::commands::init::exec(InitCommand {
        name: Some(temp_dir_name.clone()),
        path: Some(temp_dir_path.clone()),
        namespace: temp_dir_name.to_string(),
        native: false,
        absolute_paths: false,
    })
    .expect("Init command failed");

    std::env::set_current_dir(&temp_dir_path).expect("Failed to set current dir");

    let manifest = temp_dir_name.clone() + ".manifest.yaml";

    forc_index::commands::build::exec(BuildCommand {
        target: None,
        native: false,
        path: None,
        verbose: false,
        profile: None,
        release: false,
        locked: false,
        manifest: Some(manifest),
        output_dir_root: None,
    })
    .expect("Build command failed");
    let wasm_artifact_path =
        temp_dir_path.join("target/wasm32-unknown-unknown/debug/plugin_test.wasm");

    assert!(
        Path::new(&wasm_artifact_path).exists(),
        "WASM artifact not found at expected path"
    );
}

#[test]
fn new_command_initializes_project_at_new_directory() {
    let (_temp_dir, temp_dir_path, _temp_dir_name) = init_temp_dir();

    std::env::set_current_dir(&temp_dir_path).expect("Failed to set current dir");
    let new_project_name = "new_project_dir";
    let new_project_path = PathBuf::from(new_project_name);

    forc_index::commands::new::exec(NewCommand {
        name: Some(new_project_name.to_string()),
        path: new_project_path.clone(),
        namespace: new_project_name.to_string(),
        native: false,
        absolute_paths: false,
    })
    .expect("New command failed");

    std::env::set_current_dir(&new_project_path).expect("Failed to set current dir");
    let current_dir = std::env::current_dir().expect("Failed to get current dir");

    let format_path = |path: &str| format!("{}/{}", current_dir.to_string_lossy(), path);
    let manifest = format!("{}.manifest.yaml", new_project_name);
    let schema_path = format!("schema/{}.schema.graphql", new_project_name);

    [
        manifest.as_str(),
        "Cargo.toml",
        "src",
        "src/lib.rs",
        "schema",
        schema_path.as_str(),
    ]
    .iter()
    .map(|path| PathBuf::from(format_path(path)))
    .for_each(|full_path| {
        assert!(
            full_path.exists(),
            "Expected path '{}' does not exist.",
            full_path.to_string_lossy()
        )
    });
}

fn init_temp_dir() -> (TempDir, PathBuf, String) {
    let name = "plugin_test";
    let temp_dir = Builder::new()
        .prefix(name)
        .tempdir()
        .expect("Failed to create temp dir");
    let temp_dir_path = temp_dir.path().to_path_buf();

    (temp_dir, temp_dir_path, name.to_string())
}
