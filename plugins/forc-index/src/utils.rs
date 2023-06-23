use crate::{defaults, defaults::manifest_name};
use std::{fs::canonicalize, path::PathBuf, process::Command};

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
