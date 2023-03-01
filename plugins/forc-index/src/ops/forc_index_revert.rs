use crate::{cli::RevertCommand, utils::project_dir_info};
use fuel_indexer_lib::manifest::Manifest;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use tracing::{error, info};

pub async fn init(command: RevertCommand) -> anyhow::Result<()> {
    let RevertCommand {
        url,
        path,
        manifest,
        ..
    } = command;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    println!("manifest_path: {:?}", manifest_path);

    let manifest: Manifest = Manifest::from_file(manifest_path.as_path())?;

    let target = format!(
        "{}/api/index/{}/{}",
        &url, &manifest.namespace, &manifest.identifier
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        command.auth.unwrap_or_else(|| "fuel".into()).parse()?,
    );

    info!(
        "\n⬅️  Revertingindex '{}.{}' at {}",
        &manifest.namespace, &manifest.identifier, &target
    );

    let res = Client::new()
        .put(&target)
        .headers(headers)
        .send()
        .expect("Failed to deploy indexer.");

    if res.status() != StatusCode::OK {
        error!(
            "\n❌ {} returned a non-200 response code: {:?}",
            &target,
            res.status()
        );
        return Ok(());
    }

    let res_json = res
        .json::<Map<String, Value>>()
        .expect("Failed to read JSON response.");

    println!("\n{}", to_string_pretty(&res_json)?);

    info!(
        "\n✅ Successfully reverted index '{}.{}' at {} \n",
        &manifest.namespace, &manifest.identifier, &target
    );
     
    Ok(())
}
