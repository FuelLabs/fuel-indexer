use crate::IndexerService;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig},
    manifest::Manifest,
    utils::ServiceRequest,
};
use tracing::info;

#[cfg(feature = "fuel-core-lib")]
use fuel_core::service::{Config, FuelService};

#[cfg(feature = "fuel-core-lib")]
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[cfg(feature = "api-server")]
use fuel_indexer_api_server::api::GraphQlApi;

#[cfg(feature = "api-server")]
use fuel_indexer_lib::defaults::SERVICE_REQUEST_CHANNEL_SIZE;

#[cfg(feature = "api-server")]
use tokio::sync::mpsc::channel;

#[cfg(feature = "fuel-core-lib")]
async fn run_fuel_core_node() -> anyhow::Result<()> {
    let mut config = Config::local_node();
    config.addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    FuelService::new_node(config).await?;
    Ok(())
}

/// Start a local indexer service.
pub async fn exec(args: IndexerArgs) -> anyhow::Result<()> {
    let IndexerArgs { manifest, .. } = args.clone();

    let config = match &args.config {
        Some(path) => IndexerConfig::from_file(path)?,
        None => IndexerConfig::from(args.clone()),
    };

    info!("Configuration: {:?}", config);

    let (tx, rx) = if cfg!(feature = "api-server") {
        let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    #[cfg(feature = "fuel-core-lib")]
    run_fuel_core_node().await?;

    let pool = IndexerConnectionPool::connect(&config.database.to_string()).await?;

    if config.graphql_api.run_migrations {
        let mut c = pool.acquire().await?;
        queries::run_migration(&mut c).await?;
    }

    let mut service = IndexerService::new(config.clone(), pool.clone(), rx).await?;

    match manifest.map(|p| {
        info!("Using manifest file located at '{}'", p.display());
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

    if cfg!(feature = "api-server") {
        let _ = tokio::join!(service_handle, GraphQlApi::build_and_run(config, pool, tx));
    } else {
        service_handle.await?;
    };

    Ok(())
}
