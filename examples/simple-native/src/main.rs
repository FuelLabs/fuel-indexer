extern crate alloc;
use anyhow::Result;
use fuel_indexer::{
    Address, GraphQlApi, IndexerConfig, IndexerService, Manifest,
};
use fuel_indexer_lib::config::{IndexerArgs, Parser};
use fuel_indexer_macros::indexer;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;


#[indexer(
    abi = "examples/simple-native/contracts/counter/out/debug/counter-abi.json",
    namespace = "counter",
    schema = "schema/counter.graphql",
)]
mod simple_native {
    fn count_handler(event: CountEvent) {
        // Using the Count entity from the GraphQL schema
        let count = Count {
            id: event.id,
            timestamp: event.timestamp,
            count: event.count,
        };

        count.save()
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Uneccessary, but helpful tracing
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let opt = IndexerArgs::from_args();

    // Load the node config
    let config = match &opt.config {
        Some(path) => IndexerConfig::from_file(path).await?,
        None => IndexerConfig::from_opts(opt.clone()),
    };

    // Load the indexer manifest
    let manifest = Manifest::from_file(&opt.manifest.unwrap())?;

    // Create a new service to run
    let mut service = IndexerService::new(config.clone()).await?;

    // Add an indexer comprised of a list of handlers
    service
        .add_indexer(manifest, false)
        .await?;

    // Kick it off!
    let service_handle = tokio::spawn(service.run());

    // In this example, we've already started our fuel node on another process
    info!("Fuel node listening on {}", config.fuel_node.to_string());
    GraphQlApi::run(config.clone()).await;

    service_handle.await?;

    Ok(())
}
