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
        verbose,
        ..
    } = command;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

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

    if verbose {
        info!(
            "\n⬅️  Reverting indexer '{}.{}' at {}",
            &manifest.namespace, &manifest.identifier, &target
        );
    } else {
        info!("\n⬅️  Reverting indexer")
    }

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

    if verbose {
        info!(
            "\n{}\n✅ Indexer '{}'.'{}' reverted successfully.",
            to_string_pretty(&res_json)?,
            &manifest.namespace,
            &manifest.identifier
        );
    } else {
        info!("\n✅ Indexer reverted successfully.")
    }

    Ok(())
}
