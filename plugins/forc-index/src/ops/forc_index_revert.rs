use crate::ops::{forc_index_remove, forc_index_start};
use crate::{
    cli::{RevertCommand, StartCommand},
    utils::{defaults, project_dir_info},
};
use fuel_indexer_lib::{defaults as indexer_defaults, manifest::Manifest};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use tracing::info;

pub fn init(command: RevertCommand) -> anyhow::Result<()> {
    let RevertCommand { path, manifest, .. } = command;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    println!("manifest_path: {:?}", manifest_path);

    let manifest: Manifest = Manifest::from_file(manifest_path.as_path())?;

    println!("manifest: {:?}", manifest);

    let target = format!(
        "{}/api/index/{}/{}",
        &command.url, &manifest.namespace, &manifest.identifier
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        command.auth.unwrap_or_else(|| "fuel".into()).parse()?,
    );
    let delete_headers = headers.clone();

    info!(
        "\n⬅️  Removing current index '{}.{}' at {}",
        &manifest.namespace, &manifest.identifier, &target
    );

    let res = Client::new()
        .delete(&target)
        .headers(delete_headers)
        .send()
        .expect("Failed to fetch recent index.");

    if res.status() != StatusCode::OK {
        println!("Failed to remove index: {:?}", res);
        return Ok(());
    }

    let res = Client::new()
        .get(&target)
        .headers(headers)
        .send()
        .expect("Failed to fetch recent index, none exists.");

    if res.status() != StatusCode::OK {
        println!("Failed to fetch previous index: {:?}", res);
        return Ok(());
    }

    println!("GET res: {:?}", res);

    info!(
        "\n⬅️  Reverting to previous index '{}.{}' at {}",
        &manifest.namespace, &manifest.identifier, &target
    );

    Ok(())
}
