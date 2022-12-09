use anyhow::Result;
use fuel_indexer::IndexerService;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig, Parser},
    manifest::Manifest,
    utils::ServiceRequest,
};
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

    let (tx, rx) = if cfg!(feature = "api-server") {
        let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    let pool = IndexerConnectionPool::connect(&config.database.to_string()).await?;

    let mut c = pool.acquire().await.unwrap();
    queries::run_migration(&mut c).await;

    let mut service = IndexerService::new(config.clone(), pool.clone(), rx).await?;

    match opt.manifest.map(|p| {
        info!("Using bootstrap manifest file located at '{}'", p.display());
        Manifest::from_file(&p).unwrap()
    }) {
        Some(m) => {
            service.register_index_from_manifest(m).await?;
        }
        None => {
            service.register_indices_from_registry().await?;
        }
    }

    let service_handle = tokio::spawn(service.run());

    #[cfg(feature = "api-server")]
    GraphQlApi::run(config, pool, tx).await;

    service_handle.await?;

    Ok(())
}
