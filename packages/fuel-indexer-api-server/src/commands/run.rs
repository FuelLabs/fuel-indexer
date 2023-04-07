use crate::api::GraphQlApi;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{ApiServerArgs, IndexerConfig},
    utils::init_logging,
};
use tracing::info;

pub async fn exec(args: ApiServerArgs) -> anyhow::Result<()> {
    let config = match &args.config {
        Some(path) => IndexerConfig::from_file(path)?,
        None => IndexerConfig::from(args),
    };

    info!("Configuration: {:?}", config);

    let pool = IndexerConnectionPool::connect(&config.database.to_string()).await?;

    if config.run_migrations {
        let mut c = pool.acquire().await?;
        queries::run_migration(&mut c).await?;
    }

    init_logging(&config).await?;

    let _ = GraphQlApi::build_and_run(config.clone(), pool, None).await;

    Ok(())
}
