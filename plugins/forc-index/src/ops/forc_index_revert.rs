use crate::{cli::RevertCommand, utils::project_dir_info};
use fuel_indexer_lib::manifest::Manifest;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use tracing::{error, info};

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

    //@TODO change emoji
    info!(
        "\n🛑 Reverting index '{}.{}' at {}",
        &manifest.namespace, &manifest.identifier, &target
    );

    let res = Client::new()
        .get(&target)
        .headers(headers)
        .send()
        .expect("Failed to fetch the recent index");

    if res.status() != StatusCode::OK {
        error!(
            "\n❌ {} returned a non-200 response code: {:?}",
            &target,
            res.status()
        );
        return Ok(());
    }

    println!("res: {:?}", res);

    Ok(())
}
