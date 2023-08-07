use std::path::Path;
use tracing::info;

/// Given a path to a directory in which `Cargo.toml` is located, extract Cargo
/// metadata.
pub fn cargo_metadata(cargo_manifest_dir: &Path) -> anyhow::Result<serde_json::Value> {
    let output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(cargo_manifest_dir.join("Cargo.toml").as_path())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        anyhow::bail!("cargo metadata execution failed");
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    Ok(serde_json::from_str(&output_str).expect("Failed to parse JSON"))
}

/// Given a path to a directory in which `Cargo.toml` is located, find the
/// `target` directory using `cargo metadata`.
pub fn cargo_target_dir(cargo_manifest_dir: &Path) -> anyhow::Result<std::path::PathBuf> {
    let metadata_json = cargo_metadata(cargo_manifest_dir)?;

    // Use serde to extract the "target_directory" field
    let target_directory = metadata_json["target_directory"]
        .as_str()
        .expect("target_directory not found or invalid");

    Ok(target_directory.into())
}

/// Given a path to a directory in which `Cargo.toml` is located, find the
/// `workspace_root` directory using `cargo metadata`.
pub fn cargo_workspace_root_dir(
    cargo_manifest_dir: &Path,
) -> anyhow::Result<std::path::PathBuf> {
    let metadata_json = cargo_metadata(cargo_manifest_dir)?;

    // Use serde to extract the "target_directory" field
    let target_directory = metadata_json["workspace_root"]
        .as_str()
        .expect("target_directory not found or invalid");

    Ok(target_directory.into())
}

/// Set file's atime and mtime to now.
pub fn touch_file(path: &Path) -> std::io::Result<()> {
    let time = filetime::FileTime::now();
    filetime::set_file_times(path, time, time)?;
    Ok(())
}

/// Set src/lib.rs' atime and mtime to now and thus ensure the WASM module is
/// rebuilt if schema file has changed.
pub fn ensure_rebuild_if_schema_changed(
    project_dir: &Path,
    schema: &Path,
) -> std::io::Result<()> {
    let schema_mtime = {
        let metadata = std::fs::metadata(schema).unwrap();
        filetime::FileTime::from_last_modification_time(&metadata)
    };

    let lib_rs = {
        let mut path = project_dir.to_owned();
        path.push("src");
        path.push("lib.rs");
        path
    };

    let lib_rs_mtime = {
        let metadata = std::fs::metadata(lib_rs.as_path()).unwrap();
        filetime::FileTime::from_last_modification_time(&metadata)
    };

    if schema_mtime > lib_rs_mtime {
        touch_file(lib_rs.as_path())?;
    }

    Ok(())
}
