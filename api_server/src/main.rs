use anyhow::Result;
use api_server::{ApiServerArgs, ApiServerConfig, GraphQLApi, Parser};
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

    let opt = ApiServerArgs::from_args();

    let config = match &opt.config {
        Some(path) => ApiServerConfig::from_file(path).await?,
        None => ApiServerConfig::from_opts(opt.clone()),
    };

    info!("Configuration: {:?}", config);

    let api = GraphQLApi::new(config.postgres, config.graphql_api);

    let api_handle = tokio::spawn(api.run());

    api_handle.await?;

    Ok(())
}
