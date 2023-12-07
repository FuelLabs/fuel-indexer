use crate::api::WebApi;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{ApiServerArgs, IndexerConfig},
    defaults::SERVICE_REQUEST_CHANNEL_SIZE,
    utils::{init_logging, ServiceRequest},
};
use tokio::sync::mpsc::channel;
use tracing::info;

pub async fn exec(args: ApiServerArgs) -> anyhow::Result<()> {
    let config = match &args.config {
        Some(path) => IndexerConfig::from_file(path)?,
        None => IndexerConfig::from(args),
    };

    info!("Configuration: {:?}", config);

    let (tx, _) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);

    let pool = IndexerConnectionPool::connect(
        &config.database.to_string(),
        config.max_db_connections,
    )
    .await?;

    if config.run_migrations {
        let mut c = pool.acquire().await?;
        queries::run_migration(&mut c).await?;
    }

    init_logging(&config).await?;

    let _ = WebApi::build_and_run(config.clone(), pool, tx).await;

    Ok(())
}
