use forc_index::commands::{
    build::Command as BuildCommand, init::Command as InitCommand,
};
use std::fs::{self, ReadDir};
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn init_command_creates_correct_project_tree() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_dir_path = temp_dir.path().to_path_buf();
    let temp_dir_name = temp_dir_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches('.')
        .to_string();

    let command = InitCommand {
        name: Some(temp_dir_name.clone()),
        path: Some(temp_dir_path.clone()),
        namespace: temp_dir_name.to_string(),
        native: false,
        absolute_paths: false,
    };

    forc_index::commands::init::exec(command).expect("Init command failed");

    let manifest_path = temp_dir_name.clone() + ".manifest.yaml";
    let schema_path = format!("schema/{}.schema.graphql", temp_dir_name);

    let expected_project_tree = vec![
        manifest_path.as_str(),
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
    let temp_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    remove_contents_of_dir(&temp_dir_path).expect("Failed to remove output directory");

    //we set the namespace to the name of the dir so that the
    //build command can find the manifest
    let command = InitCommand {
        name: Some("output".to_string()),
        path: Some(temp_dir_path.clone()),
        namespace: "output".to_string(),
        native: false,
        absolute_paths: false,
    };

    forc_index::commands::init::exec(command).expect("Init command failed");

    assert!(
        Path::new(&temp_dir_path.join("output.manifest.yaml")).exists(),
        "Manifest not found at expected path"
    );

    println!("init success");

    //cd into the output dir
    std::env::set_current_dir(&temp_dir_path).expect("Failed to set current dir");

    let command = BuildCommand {
        target: None,
        native: false,
        path: None,
        verbose: true,
        profile: None,
        release: false,
        locked: false,
        manifest: None,
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
