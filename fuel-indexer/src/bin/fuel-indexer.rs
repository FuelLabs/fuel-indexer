use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use fuel_core::service::{Config, FuelService};
use fuel_indexer::{
    config::{IndexerArgs, Parser},
    GraphQlApi, IndexerConfig, IndexerService, Manifest,
};
use fuel_indexer_schema::db::run_migration;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

#[tokio::main]
pub async fn main() -> Result<()> {
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let opt = IndexerArgs::from_args();

    let mut config = match &opt.config {
        Some(path) => IndexerConfig::from_file(path).await?,
        None => IndexerConfig::from_opts(opt.clone()),
    };

    info!("Configuration: {:?}", config);

    run_migration(&config.database.to_string()).await;

    let _local_node = if opt.local {
        let s = FuelService::new_node(Config::local_node())
            .await
            .expect("Failed to start node using local config");
        config.fuel_node = s.bound_address.into();
        Some(s)
    } else {
        None
    };

    info!(
        "Subscribing to Fuel node at {}",
        &config.fuel_node.to_string()
    );

    let mut service = IndexerService::new(config.clone()).await?;

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
    GraphQlApi::run(config).await;

    service_handle.await?;

    Ok(())
}
