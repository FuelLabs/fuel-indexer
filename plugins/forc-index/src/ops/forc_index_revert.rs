use crate::{cli::RevertCommand, utils::project_dir_info};
use fuel_indexer_database::{queries, types::IndexAssetType, IndexerConnectionPool};
use fuel_indexer_lib::manifest::Manifest;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use tracing::info;

pub async fn init(command: RevertCommand) -> anyhow::Result<()> {
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

    info!(
        "\n⬅️  Reverting to previous index '{}.{}' at {}",
        &manifest.namespace, &manifest.identifier, &target
    );

    let db_url = "postgres://postgres@127.0.0.1";
    let pool = IndexerConnectionPool::connect(&db_url).await?;
    let mut conn = pool.acquire().await?;

    let index_id =
        queries::index_id_for(&mut conn, &manifest.namespace, &manifest.identifier)
            .await?;

    let asset =
        queries::latest_asset_for_index(&mut conn, &index_id, IndexAssetType::Wasm)
            .await?;

    Ok(())
}
