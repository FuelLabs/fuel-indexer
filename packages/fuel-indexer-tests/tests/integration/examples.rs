use forc_index::commands::start::Command as StartCommand;
use fuel_indexer_tests::defaults;
use std::process::Command;
use tokio::time::{sleep, Duration};

#[actix_web::test]
#[cfg(all(feature = "examples"))]
async fn test_release_build_hello_world_wasm_artifact() {
    //test the command in our documentation: Examples
    let output = Command::new("cargo")
        .args(&[
            "build",
            "-p",
            "hello_indexer",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
}

#[actix_web::test]
#[cfg(all(feature = "examples"))]
async fn test_start_indexer_with_hello_world() {
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    let project_root = std::env::var("PROJECT_ROOT").expect("Failed to get PROJECT_ROOT");
    std::env::set_current_dir(project_root).expect("Failed to set current dir");

    let entries = std::fs::read_dir("./").expect("Failed to read dir");
    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        if entry.file_type().expect("Failed to get file type").is_dir() {
            println!("{}", entry.path().display());
        }
    }

    let manifest = format!(
        "{}/{}.manifest.yaml",
        defaults::HELLO_WORLD_PATH,
        defaults::HELLO_WORLD_INDEXER
    );

    let entries = std::fs::read_dir("examples/block-explorer/explorer-indexer/")
        .expect("Failed to read dir");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        println!("{}", entry.path().display());
    }

    println!("Manifest: {}", manifest);

    let start = forc_index::commands::start::exec(Box::new(StartCommand {
        manifest: Some(manifest.into()),
        ..Default::default()
    }))
    .await;

    sleep(Duration::from_secs(1)).await;
    assert!(start.is_ok());

    std::env::set_current_dir(original_dir).expect("Failed to set current dir");
}

#[actix_web::test]
#[cfg(all(feature = "examples"))]
async fn test_release_build_block_explorer_wasm_artifact() {
    let output = Command::new("cargo")
        .args(&[
            "build",
            "-p",
            "explorer_indexer",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
}

#[actix_web::test]
#[cfg(all(feature = "examples"))]
async fn test_start_indexer_block_explorer() {
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    let project_root = std::env::var("PROJECT_ROOT").expect("Failed to get PROJECT_ROOT");
    std::env::set_current_dir(project_root).expect("Failed to set current dir");

    let entries = std::fs::read_dir("./").expect("Failed to read dir");
    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        if entry.file_type().expect("Failed to get file type").is_dir() {
            println!("{}", entry.path().display());
        }
    }

    let manifest = format!(
        "{}/{}.manifest.yaml",
        defaults::BLOCK_EXPLORER_PATH,
        defaults::BLOCK_EXPLORER_INDEXER
    );

    let entries = std::fs::read_dir("examples/block-explorer/explorer-indexer/")
        .expect("Failed to read dir");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        println!("{}", entry.path().display());
    }

    println!("Manifest: {}", manifest);

    let start = forc_index::commands::start::exec(Box::new(StartCommand {
        manifest: Some(manifest.into()),
        ..Default::default()
    }))
    .await;

    sleep(Duration::from_secs(1)).await;
    assert!(start.is_ok());
    std::env::set_current_dir(original_dir).expect("Failed to set current dir");
}
