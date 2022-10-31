use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use fuel_indexer::IndexerService;
#[cfg(feature = "api-server")]
use fuel_indexer_api_server::api::GraphQlApi;
use fuel_indexer_database::queries;
use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig, Parser},
    defaults::ASSET_REFRESH_CHANNEL_SIZE,
    manifest::Manifest,
    utils::AssetReloadRequest,
};

use tokio::sync::mpsc;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

#[tokio::main]
pub async fn main() -> Result<()> {
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => {
            EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
        }
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let opt = IndexerArgs::from_args();

    let config = match &opt.config {
        Some(path) => IndexerConfig::from_file(path)?,
        None => IndexerConfig::from_opts(opt.clone()),
    };

    info!("Configuration: {:?}", config);

    queries::run_migration(&config.database.to_string()).await;

    info!(
        "Subscribing to Fuel node at {}",
        &config.fuel_node.to_string()
    );

    let (tx, rx) = mpsc::channel::<AssetReloadRequest>(ASSET_REFRESH_CHANNEL_SIZE);
    let mut service = IndexerService::new(config.clone(), Some(rx)).await?;

    let mut manifest: Option<Manifest> = None;

    if opt.manifest.is_some() {
        let path = opt.manifest.expect("Could not get path from manifest");

        info!(
            "Using bootstrap manifest file located at '{}'",
            path.display()
        );

        let mut file = File::open(&path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let local_manifest: Manifest = serde_yaml::from_str(&contents)?;
        manifest = Some(local_manifest);
    }

    service.register_indices(manifest, false).await?;

    let service_handle = tokio::spawn(service.run());
    GraphQlApi::run(config, Some(tx)).await;

    service_handle.await?;

    Ok(())
}
