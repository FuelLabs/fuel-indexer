use crate::{cli::RemoveCommand, utils::project_dir_info};
use fuel_indexer_lib::manifest::Manifest;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use tracing::{error, info};

pub fn init(command: RemoveCommand) -> anyhow::Result<()> {
    let RemoveCommand {
        path,
        manifest,
        url,
        auth,
        verbose,
        ..
    } = command;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    let manifest: Manifest = Manifest::from_file(manifest_path.as_path())?;

    let target = format!(
        "{url}/api/index/{}/{}",
        &manifest.namespace, &manifest.identifier
    );

    let mut headers = HeaderMap::new();
    if let Some(auth) = auth {
        headers.insert(AUTHORIZATION, auth.parse()?);
    }

    if verbose {
        info!(
            "\n🛑 Removing index '{}.{}' at {target}",
            &manifest.namespace, &manifest.identifier
        );
    } else {
        info!("\n🛑 Removing index ")
    }

    let res = Client::new()
        .delete(&target)
        .headers(headers)
        .send()
        .expect("Failed to remove index.");

    if res.status() != StatusCode::OK {
        error!(
            "\n❌ {target} returned a non-200 response code: {:?}",
            res.status()
        );
        return Ok(());
    }

    let res_json = res
        .json::<Map<String, Value>>()
        .expect("Failed to read JSON response.");

    info!("\n{}", to_string_pretty(&res_json)?);

    if verbose {
        info!(
            "\n✅ Successfully removed index '{}.{}' at {} \n",
            &manifest.namespace, &manifest.identifier, &target
        );
    } else {
        info!("\n✅ Successfully removed index \n");
    }

    Ok(())
}
