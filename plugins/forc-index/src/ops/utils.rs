use serde_yaml::Value;
use std::path::Path;

pub fn cargo_target_dir(
    cargo_manifest_path: &Path,
) -> anyhow::Result<std::path::PathBuf> {
    let output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(cargo_manifest_path.join("Cargo.toml").as_path())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        anyhow::bail!("cargo metadata execution failed");
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    let metadata_json: Value =
        serde_json::from_str(&output_str).expect("Failed to parse JSON");

    // Use serde to extract the "target_directory" field
    let target_directory = metadata_json["target_directory"]
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

/// Set src/lib.rs' atime and mtime to now.
pub fn touch_lib_rs(project_dir: &Path, schema: &Path) -> std::io::Result<()> {
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