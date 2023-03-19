use forc_index::cli::{run_cli, ForcIndex, Opt};
use forc_index::commands::build::Command as BuildCommand;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_build_command_creates_artifact() -> anyhow::Result<()> {
    let output_dir = TempDir::new()?;
    let output_dir_path = output_dir.path().to_path_buf();

    let manifest_path = PathBuf::from("tests/assets/test_indexer.manifest.yaml");

    let command = BuildCommand {
        target: None,
        native: false,
        path: Some(output_dir_path.clone()),
        verbose: false,
        profile: None,
        release: false,
        locked: false,
        manifest: Some(manifest_path.clone()),
        output_dir_root: Some(output_dir_path.clone()),
    };
    let opt = Opt { command };

    let result = run_cli();

    assert_eq!(1, 0);

    Ok(())
}
