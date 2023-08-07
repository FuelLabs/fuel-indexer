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
