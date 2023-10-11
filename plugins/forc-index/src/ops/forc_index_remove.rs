use crate::{cli::RemoveCommand, utils::project_dir_info};
use fuel_indexer_lib::manifest::Manifest;
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    Client, StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use tracing::{error, info};

pub async fn init(command: RemoveCommand) -> anyhow::Result<()> {
    let RemoveCommand {
        path,
        manifest,
        url,
        auth,
        verbose,
        ..
    } = command;

    let (_root_dir, manifest_path, _indexer_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    let manifest: Manifest = Manifest::from_file(manifest_path.as_path())?;

    let target = format!(
        "{url}/api/index/{}/{}",
        manifest.namespace(),
        manifest.identifier()
    );

    let mut headers = HeaderMap::new();
    if let Some(auth) = auth {
        headers.insert(AUTHORIZATION, auth.parse()?);
    }

    if verbose {
        info!(
            "\n🛑 Removing indexer '{}.{}' at {target}",
            manifest.namespace(),
            manifest.identifier()
        );
    } else {
        info!("\n🛑 Removing indexer.")
    }

    let res = Client::new()
        .delete(&target)
        .headers(headers)
        .send()
        .await
        .expect("Failed to remove indexer.");

    let status = res.status();
    let res_json = res
        .json::<Map<String, Value>>()
        .await
        .expect("Failed to read JSON response.");

    if status != StatusCode::OK {
        if verbose {
            error!("\n❌ {target} returned a non-200 response code: {status:?}",);

            info!("\n{}", to_string_pretty(&res_json)?);
        } else {
            info!("\n{}", to_string_pretty(&res_json)?);
        }

        return Ok(());
    }

    if verbose {
        info!(
            "\n{}\n✅ Successfully removed indexer '{}.{}' at {target} \n",
            to_string_pretty(&res_json)?,
            manifest.namespace(),
            manifest.identifier()
        );
    } else {
        info!("\n✅ Successfully removed indexer\n");
    }

    Ok(())
}
