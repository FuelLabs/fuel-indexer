use forc_index::commands::build::Command as BuildCommand;
use std::path::{Path, PathBuf};

#[test]
fn test_build_command_creates_artifact() {
    let output_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests");

    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("plugin_test.manifest.yaml");

    println!("Output dir path: {:?}", output_dir_path);
    println!("Manifest path: {:?}", manifest_path);

    let command = BuildCommand {
        target: None,
        native: false,
        path: Some(output_dir_path.clone()),
        verbose: false,
        profile: None,
        release: false,
        locked: false,
        manifest: Some(manifest_path.to_str().unwrap().to_owned()),
        output_dir_root: Some(output_dir_path.clone()),
    };

    forc_index::commands::build::exec(command).expect("Build command failed");
    let wasm_artifact_path =
        output_dir_path.join("target/wasm32-unknown-unknown/debug/plugin_test.wasm");

    assert!(
        Path::new(&wasm_artifact_path).exists(),
        "WASM artifact not found at expected path"
    );
}
