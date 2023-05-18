pub mod defaults;
pub(crate) mod restricted;

use anyhow::{bail, Result};
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

/// Simple function to convert kebab-case to snake_case.
pub(crate) fn kebab_to_snake_case(s: &str) -> String {
    s.replace('-', "_")
}

pub fn validate_name(name: &str, use_case: &str) -> Result<()> {
    // if true returns formatted error
    restricted::contains_invalid_char(name, use_case)?;

    if restricted::is_keyword(name) {
        bail!("the name `{name}` cannot be used as a package name, it is a Sway keyword");
    }
    if restricted::is_conflicting_artifact_name(name) {
        bail!(
            "the name `{name}` cannot be used as a package name, \
            it conflicts with Forc's build directory names"
        );
    }
    if name.to_lowercase() == "test" {
        bail!(
            "the name `test` cannot be used as a project name, \
            it conflicts with Sway's built-in test library"
        );
    }
    if restricted::is_conflicting_suffix(name) {
        bail!(
            "the name `{name}` is part of Sway's standard library\n\
            It is recommended to use a different name to avoid problems."
        );
    }
    if restricted::is_windows_reserved(name) {
        if cfg!(windows) {
            bail!("cannot use name `{name}`, it is a reserved Windows filename");
        } else {
            bail!(
                "the name `{name}` is a reserved Windows filename\n\
                This package will not work on Windows platforms."
            );
        }
    }
    if restricted::is_non_ascii_name(name) {
        bail!("the name `{name}` contains non-ASCII characters which are unsupported");
    }
    Ok(())
}
