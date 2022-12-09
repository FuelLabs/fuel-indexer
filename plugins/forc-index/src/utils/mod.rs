pub mod defaults;

pub(crate) fn dasherize_to_underscore(s: &str) -> String {
    str::replace(s, "-", "_")
}

pub(crate) fn extract_manifest_fields(
    manifest: serde_yaml::Value,
) -> anyhow::Result<(String, String, String, String)> {
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
    let graphql_schema: String = manifest
        .get(&serde_yaml::Value::String("graphql_schema".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let module: serde_yaml::Value = manifest
        .get(&serde_yaml::Value::String("module".into()))
        .unwrap()
        .to_owned();
    let module_path: String = module
        .get(&serde_yaml::Value::String("wasm".into()))
        .unwrap_or_else(|| {
            module
                .get(&serde_yaml::Value::String("native".into()))
                .unwrap()
        })
        .as_str()
        .unwrap()
        .to_string();

    Ok((namespace, identifier, graphql_schema, module_path))
}
