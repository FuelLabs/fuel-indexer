use anyhow::Result;
use fuel_indexer::{GraphQlApi, IndexerArgs, IndexerConfig, Parser};
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

    let opts = IndexerArgs::from_args();

    let config = match &opts.config {
        Some(path) => IndexerConfig::from_file(path).await?,
        None => IndexerConfig::from_opts(opts.clone()),
    };

    info!("Configuration: {:?}", config);

    let api_handle = tokio::spawn(GraphQlApi::run(config.clone()));

    api_handle.await?;

    Ok(())
}
