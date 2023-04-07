pub mod defaults;
use defaults::manifest_name;
use std::{fs::canonicalize, path::PathBuf};

pub(crate) fn dasherize_to_underscore(s: &str) -> String {
    str::replace(s, "-", "_")
}

pub(crate) fn project_dir_info(
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

pub(crate) fn default_manifest_filename(name: &str) -> String {
    format!("{name}.manifest.yaml")
}

pub(crate) fn default_schema_filename(name: &str) -> String {
    format!("{name}.schema.graphql")
}
