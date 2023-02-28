use crate::{cli::RevertCommand, utils::project_dir_info};
use fuel_indexer_database::{queries, types::IndexAssetType, IndexerConnectionPool};
use fuel_indexer_lib::manifest::Manifest;
use reqwest::{header::{HeaderMap, AUTHORIZATION}, blocking::multipart::Form};

pub async fn init(command: RevertCommand) -> anyhow::Result<()> {
    let RevertCommand { url, path, manifest, .. } = command;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    println!("manifest_path: {:?}", manifest_path);

    let manifest: Manifest = Manifest::from_file(manifest_path.as_path())?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        command.auth.unwrap_or_else(|| "fuel".into()).parse()?,
    );

    let db_url = "postgres://postgres@127.0.0.1";
    let pool = IndexerConnectionPool::connect(&db_url).await?;
    let mut conn = pool.acquire().await?;

    let index_id =
        queries::index_id_for(&mut conn, &manifest.namespace, &manifest.identifier)
            .await?;

    let asset =
        queries::penultimate_asset_for_index(&mut conn, &index_id, IndexAssetType::Wasm)
            .await?;

    let target = format!("{}/api/index/{}/{}", &url, &manifest.namespace, &manifest.identifier);

    let form = Form::new()
        .file("manifest", &manifest_path)?
        .file("schema", graphql_schema)?
        .file("wasm", module_path)?;

    let res = Client::new()
        .post(&target)
        .multipart(form)
        .headers(headers)
        .send()
        .expect("Failed to deploy indexer.");

    Ok(())
}
