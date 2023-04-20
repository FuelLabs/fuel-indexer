use crate::IndexerService;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig},
    manifest::Manifest,
    utils::{init_logging, ServiceRequest},
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
async fn run_fuel_core_node() -> anyhow::Result<FuelService> {
    // TODO: This should accept what ever is in FuelNodeConfig
    let config = Config {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000),
        ..Config::local_node()
    };
    let srvc = FuelService::new_node(config).await?;
    Ok(srvc)
}

pub async fn exec(args: IndexerArgs) -> anyhow::Result<()> {
    let IndexerArgs { manifest, .. } = args.clone();

    let config = args
        .clone()
        .config
        .map(IndexerConfig::from_file)
        .unwrap_or(Ok(IndexerConfig::from(args)))?;

    init_logging(&config).await?;

    info!("Configuration: {:?}", config);

    let (tx, rx) = if cfg!(feature = "api-server") {
        let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    let pool = IndexerConnectionPool::connect(&config.database.to_string()).await?;

    if config.run_migrations {
        let mut c = pool.acquire().await?;
        queries::run_migration(&mut c).await?;
    }

    let mut service = IndexerService::new(config.clone(), pool.clone(), rx).await?;

    match manifest.map(|p| {
        info!("Using manifest file located at '{}'", p.display());
        Manifest::from_file(&p).unwrap()
    }) {
        Some(m) => {
            info!(
                "✨ ✨ GraphQL Playground at: http://localhost:29987/api/playground/{}/{}",
                m.namespace, m.identifier
            );
            service.register_index_from_manifest(m).await?;
        }
        None => {
            service.register_indices_from_registry().await?;
        }
    }

    let service_handle = tokio::spawn(service.run());

    if cfg!(feature = "api-server") {
        let gql_handle =
            tokio::spawn(GraphQlApi::build_and_run(config.clone(), pool, tx));

        #[cfg(feature = "fuel-core-lib")]
        if config.local_fuel_node {
            let fuel_node_handle = tokio::spawn(run_fuel_core_node());
            let _ = tokio::join!(service_handle, gql_handle, fuel_node_handle);
        }

        #[cfg(not(feature = "fuel-core-lib"))]
        let _ = tokio::join!(service_handle, gql_handle);
    } else {
        #[cfg(feature = "fuel-core-lib")]
        if config.local_fuel_node {
            let fuel_node_handle = tokio::spawn(run_fuel_core_node());
            let _ = tokio::join!(service_handle, fuel_node_handle);
        }

        #[cfg(not(feature = "fuel-core-lib"))]
        service_handle.await?
    };

    Ok(())
}
