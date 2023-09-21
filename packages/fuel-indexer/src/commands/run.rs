use crate::IndexerService;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig},
    defaults,
    manifest::Manifest,
    utils::{init_logging, ServiceRequest},
};
use tokio::signal::unix::{signal, Signal, SignalKind};
use tokio::sync::mpsc::channel;
use tracing::info;

#[cfg(feature = "api-server")]
use fuel_indexer_api_server::api::WebApi;

// Returns a future which completes when a shutdown signal has been received.
fn shutdown_signal_handler() -> std::io::Result<impl futures::Future<Output = ()>> {
    let mut sighup: Signal = signal(SignalKind::hangup())?;
    let mut sigterm: Signal = signal(SignalKind::terminate())?;
    let mut sigint: Signal = signal(SignalKind::interrupt())?;

    let future = async move {
        #[cfg(unix)]
        {
            tokio::select! {
                _ = sighup.recv() => {
                    info!("Received SIGHUP. Stopping services.");
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM. Stopping services.");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT. Stopping services.");
                }
            }
        }

        #[cfg(not(unix))]
        {
            signal::ctrl_c().await?;
            info!("Received CTRL+C. Stopping services.");
        }
    };

    Ok(future)
}

pub async fn exec(args: IndexerArgs) -> anyhow::Result<()> {
    let mut subsystems: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();

    subsystems.spawn(shutdown_signal_handler()?);

    let IndexerArgs {
        manifest,
        embedded_database,
        postgres_database,
        postgres_password,
        postgres_port,
        postgres_user,
        remove_data,
        ..
    } = args.clone();

    let args_config = args.config.clone();

    if embedded_database {
        let name = postgres_database
            .clone()
            .unwrap_or(defaults::POSTGRES_DATABASE.to_string());
        let password = postgres_password
            .clone()
            .unwrap_or(defaults::POSTGRES_PASSWORD.to_string());
        let user = postgres_user
            .clone()
            .unwrap_or(defaults::POSTGRES_USER.to_string());
        let port = postgres_port
            .clone()
            .unwrap_or(defaults::POSTGRES_PORT.to_string());

        let create_db_cmd = forc_postgres::cli::CreateDbCommand {
            name,
            password,
            user,
            port,
            persistent: true,
            config: args_config.clone(),
            start: true,
            ..Default::default()
        };

        forc_postgres::commands::create::exec(create_db_cmd).await?;
    }

    let config = args
        .config
        .clone()
        .map(IndexerConfig::from_file)
        .unwrap_or(Ok(IndexerConfig::from(args)))?;

    init_logging(&config).await?;

    info!("Configuration: {:?}", config);

    #[allow(unused)]
    let (tx, rx) = channel::<ServiceRequest>(defaults::SERVICE_REQUEST_CHANNEL_SIZE);

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
                    "✨ ✨ GraphQL Playground at: http://localhost:29987/api/playground/{}/{}", m.namespace(), m.identifier()
                );
            service
                .register_indexer_from_manifest(m, remove_data)
                .await?;
        }
        None => {
            service.register_indexers_from_registry().await?;
        }
    }

    subsystems.spawn(async {
        let result = service.run().await;
        if let Err(e) = result {
            tracing::error!("Indexer Service failed: {e}");
        }
    });

    #[cfg(feature = "api-server")]
    subsystems.spawn({
        let config = config.clone();
        let pool = pool.clone();
        async {
            if let Err(e) = WebApi::build_and_run(config, pool, tx).await {
                tracing::error!("Api Server failed: {e}");
            }
        }
    });

    if config.enable_block_store {
        subsystems.spawn(crate::service::create_block_sync_task(
            config.clone(),
            pool.clone(),
        ));
    };

    #[cfg(feature = "fuel-core-lib")]
    {
        use fuel_core::service::{Config, FuelService};
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        if config.local_fuel_node {
            let config = Config {
                addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000),
                ..Config::local_node()
            };
            subsystems.spawn(async move {
                if let Err(e) = FuelService::new_node(config).await {
                    tracing::error!("Fuel Node failed: {e}");
                };
            });
        }
    };

    // Each subsystem runs its own loop, and we require all subsystems for the
    // Indexer service to operate correctly. If any of the subsystems stops
    // running, the entire Indexer Service exits.
    if subsystems.join_next().await.is_some() {
        subsystems.shutdown().await;
    }

    if embedded_database {
        let name = postgres_database.unwrap_or(defaults::POSTGRES_DATABASE.to_string());

        let stop_db_cmd = forc_postgres::cli::StopDbCommand {
            name,
            config: args_config.clone(),
            database_dir: None,
            verbose: false,
        };

        forc_postgres::commands::stop::exec(stop_db_cmd).await?;
    };

    Ok(())
}
