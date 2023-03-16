pub mod defaults;
pub mod log;
use defaults::manifest_name;
use std::{fs::canonicalize, path::PathBuf};

pub(crate) fn dasherize_to_underscore(s: &str) -> String {
    str::replace(s, "-", "_")
}

pub(crate) fn extract_manifest_fields(
    manifest: serde_yaml::Value,
    output_dir_root: Option<&PathBuf>,
) -> anyhow::Result<(String, String, PathBuf, PathBuf)> {
    let namespace: String = manifest
        .get(&serde_yaml::Value::String("namespace".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let identifier: String = manifest
        .get(&serde_yaml::Value::String("identifier".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let mut graphql_schema = PathBuf::from(
        manifest
            .get(&serde_yaml::Value::String("graphql_schema".into()))
            .unwrap()
            .as_str()
            .unwrap(),
    );

    let module: serde_yaml::Value = manifest
        .get(&serde_yaml::Value::String("module".into()))
        .unwrap()
        .to_owned();

    let mut module_path = PathBuf::from(
        module
            .get(&serde_yaml::Value::String("wasm".into()))
            .unwrap_or_else(|| {
                module
                    .get(&serde_yaml::Value::String("native".into()))
                    .unwrap()
            })
            .as_str()
            .unwrap(),
    );

    if let Some(output_dir_root) = output_dir_root {
        graphql_schema = output_dir_root.join(graphql_schema);
        module_path = output_dir_root.join(module_path);
    }

    Ok((namespace, identifier, graphql_schema, module_path))
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
