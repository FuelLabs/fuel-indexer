use forc_index::commands::{
    build::Command as BuildCommand, init::Command as InitCommand,
};
use std::fs::{self, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

#[test]
fn init_command_creates_correct_project_tree() {
    let test_indexer_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    remove_contents_of_dir(&test_indexer_path)
        .expect("Failed to remove output directory");

    let command = InitCommand {
        name: Some("plugin_test".to_string()),
        path: Some(test_indexer_path.clone()),
        namespace: "plugin_test".to_string(),
        native: false,
        absolute_paths: true,
    };

    forc_index::commands::init::exec(command).expect("Init command failed");

    let expected_project_tree = vec![
        "plugin_test.manifest.yaml",
        "Cargo.toml",
        "src",
        "src/lib.rs",
        "schema",
        "schema/plugin_test.schema.graphql",
    ];

    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");

    for path in expected_project_tree {
        let full_path = base_path.join(path);
        assert!(
            full_path.exists(),
            "Expected path '{}' does not exist.",
            full_path.to_string_lossy()
        )
    }

    remove_contents_of_dir(&test_indexer_path)
        .expect("Failed to remove output directory");
}

#[test]
fn build_command_creates_artifact_at_expected_path() {
    let test_indexer_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    remove_contents_of_dir(&test_indexer_path)
        .expect("Failed to remove output directory");

    let command = InitCommand {
        name: Some("plugin_test".to_string()),
        path: Some(test_indexer_path.clone()),
        namespace: "plugin_test".to_string(),
        native: false,
        absolute_paths: true,
    };

    forc_index::commands::init::exec(command).expect("Init command failed");

    let output_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    println!("Wasm pth: {:?}", wasm_artifact_path);

    assert!(
        Path::new(&wasm_artifact_path).exists(),
        "WASM artifact not found at expected path"
    );
}

fn remove_contents_of_dir(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        let entries: ReadDir = fs::read_dir(path)?;
        entries.for_each(|entry| {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Err(e) = fs::remove_dir_all(&entry_path) {
                        eprintln!("Failed to remove directory: {:?}", e);
                    }
                } else {
                    if let Err(e) = fs::remove_file(&entry_path) {
                        eprintln!("Failed to remove file: {:?}", e);
                    }
                }
            }
        });
    }
    Ok(())
}
