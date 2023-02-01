use fuel_indexer_api_server::api::GraphQlApi;
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::config::{ApiServerArgs, IndexerConfig};
use tracing::info;

pub async fn exec(args: ApiServerArgs) -> anyhow::Result<()> {
    let config = match &args.config {
        Some(path) => IndexerConfig::from_file(path)?,
        None => IndexerConfig::from(args),
    };

    info!("Configuration: {:?}", config);

    let pool = IndexerConnectionPool::connect(&config.database.to_string()).await?;

    let _ = GraphQlApi::build_and_run(config.clone(), pool, None).await;

    Ok(())
}
