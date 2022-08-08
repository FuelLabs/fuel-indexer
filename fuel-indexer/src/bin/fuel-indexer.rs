use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use fuel_core::service::{Config, FuelService};
use fuel_indexer::{GraphQlApi, IndexerConfig, IndexerService, Manifest};
use fuel_indexer_lib::config::{IndexerArgs, Parser};
use fuel_indexer_schema::db::run_migration;
use tokio::join;
use tracing::{error, info};
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

    run_migration(&config.database_config.to_string()).await;

    let _local_node = if opt.local {
        let s = FuelService::new_node(Config::local_node()).await.unwrap();
        config.fuel_node = s.bound_address.into();
        Some(s)
    } else {
        None
    };

    info!(
        "Subscribing to Fuel node at {}",
        &config.fuel_node.to_string()
    );
    let api_handle = tokio::spawn(GraphQlApi::run(config.clone()));

    let mut service = IndexerService::new(config).await?;

    // TODO: need an API endpoint to upload/create these things.....
    if opt.test_manifest.is_some() {
        let path = opt.test_manifest.unwrap();

        let mut file = File::open(&path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let manifest: Manifest = serde_yaml::from_str(&contents)?;

        service.add_wasm_indexer(manifest, false).await?;
    }

    let service_handle = tokio::spawn(service.run());

    let (first, second) = join!(api_handle, service_handle);

    if let Err(e) = first {
        error!("{:?}", e)
    }
    if let Err(e) = second {
        error!("{:?}", e)
    }
    Ok(())
}
