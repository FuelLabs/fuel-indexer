use crate::{defaults, defaults::manifest_name};
use fuel_indexer_lib::ExecutionSource;
use reqwest::{multipart::Part, Body};
use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
    process::Command,
};
use tokio::fs::File;
use tokio::io;

pub fn dasherize_to_underscore(s: &str) -> String {
    str::replace(s, "-", "_")
}

pub fn project_dir_info(
    path: Option<&PathBuf>,
    manifest: Option<&String>,
) -> anyhow::Result<(PathBuf, PathBuf, String)> {
    let curr = std::env::current_dir()?;
    let root = canonicalize(path.unwrap_or(&curr))?;
    let name = root.file_name().unwrap().to_str().unwrap().to_string();
    let mani_name = dasherize_to_underscore(&manifest_name(&name));
    let manifest = root.join(manifest.unwrap_or(&mani_name));
    Ok((root, manifest, name))
}

pub fn default_manifest_filename(name: &str) -> String {
    format!("{name}.manifest.yaml")
}

pub fn default_schema_filename(name: &str) -> String {
    format!("{name}.schema.graphql")
}

pub fn find_executable_with_msg(exec_name: &str) -> (String, Option<String>, String) {
    let (emoji, path) = find_executable(exec_name);
    let p = path.clone();
    (emoji, path, format_exec_msg(exec_name, p))
}

pub fn format_exec_msg(exec_name: &str, path: Option<String>) -> String {
    if let Some(path) = path {
        rightpad_whitespace(&path, defaults::MESSAGE_PADDING)
    } else {
        rightpad_whitespace(
            &format!("Can't locate {exec_name}."),
            defaults::MESSAGE_PADDING,
        )
    }
}

pub fn find_executable(exec_name: &str) -> (String, Option<String>) {
    match Command::new("which").arg(exec_name).output() {
        Ok(o) => {
            let path = String::from_utf8_lossy(&o.stdout)
                .strip_suffix('\n')
                .map(|x| x.to_string())
                .unwrap_or_else(String::new);

            if !path.is_empty() {
                (
                    center_align("✅", defaults::SUCCESS_EMOJI_PADDING),
                    Some(path),
                )
            } else {
                (center_align("⛔️", defaults::FAIL_EMOJI_PADDING - 2), None)
            }
        }
        Err(_e) => (center_align("⛔️", defaults::FAIL_EMOJI_PADDING), None),
    }
}

pub fn center_align(s: &str, n: usize) -> String {
    format!("{s: ^n$}")
}

pub fn rightpad_whitespace(s: &str, n: usize) -> String {
    format!("{s:0n$}")
}

pub async fn file_part<T: AsRef<Path>>(path: T) -> io::Result<Part> {
    let path = path.as_ref();
    let file_name = path
        .file_name()
        .map(|filename| filename.to_string_lossy().into_owned());
    let file = File::open(path).await?;
    let field = Part::stream(Body::from(file));

    Ok(if let Some(file_name) = file_name {
        field.file_name(file_name)
    } else {
        field
    })
}

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
pub fn ensure_rebuild_if_schema_or_manifest_changed(
    project_dir: &Path,
    schema: &Path,
    manifest: &Path,
    exec_source: ExecutionSource,
) -> std::io::Result<()> {
    let schema_mtime = {
        let metadata = std::fs::metadata(schema).unwrap();
        filetime::FileTime::from_last_modification_time(&metadata)
    };

    let manifest_mtime = {
        let metadata = std::fs::metadata(manifest).unwrap();
        filetime::FileTime::from_last_modification_time(&metadata)
    };

    let entrypoint_rs = {
        let sourcefile = match exec_source {
            ExecutionSource::Native => "main.rs",
            ExecutionSource::Wasm => "lib.rs",
        };
        let mut path = project_dir.to_owned();
        path.push("src");
        path.push(sourcefile);
        path
    };

    let entrypoint_rs_mtime = {
        let metadata = std::fs::metadata(entrypoint_rs.as_path()).unwrap();
        filetime::FileTime::from_last_modification_time(&metadata)
    };

    if schema_mtime > entrypoint_rs_mtime || manifest_mtime > entrypoint_rs_mtime {
        touch_file(entrypoint_rs.as_path())?;
    }

    Ok(())
}
