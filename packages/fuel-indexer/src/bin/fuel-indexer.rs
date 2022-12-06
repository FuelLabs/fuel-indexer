use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use fuel_indexer::IndexerService;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig, Parser},
    manifest::Manifest,
    utils::ServiceRequest,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

#[cfg(feature = "api-server")]
use fuel_indexer_api_server::api::GraphQlApi;

#[cfg(feature = "api-server")]
use fuel_indexer_lib::defaults::SERVICE_REQUEST_CHANNEL_SIZE;

#[cfg(feature = "api-server")]
use tokio::sync::mpsc::channel;

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

    #[allow(unused)]
    let (tx, rx): (
        Option<Sender<ServiceRequest>>,
        Option<Receiver<ServiceRequest>>,
    ) = match () {
        #[cfg(feature = "api-server")]
        () => {
            let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);
            (Some(tx), Some(rx))
        }
        () => (None, None),
    };

    let pool = IndexerConnectionPool::connect(&config.database.to_string())
        .await
        .expect("Failed to open connection pool");

    let mut service = IndexerService::new(config.clone(), pool.clone(), rx).await?;

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

    service.register_indices(manifest).await?;

    let service_handle = tokio::spawn(service.run());

    #[cfg(feature = "api-server")]
    GraphQlApi::run(config, Some(pool.clone()), tx).await;

    service_handle.await?;

    Ok(())
}
