use crate::IndexerService;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::{
    config::{IndexerArgs, IndexerConfig},
    defaults::SERVICE_REQUEST_CHANNEL_SIZE,
    manifest::Manifest,
    utils::{init_logging, ServiceRequest},
};
use tokio::sync::mpsc::channel;
use tracing::info;

#[cfg(feature = "api-server")]
use fuel_indexer_api_server::api::GraphQlApi;

pub async fn exec(args: IndexerArgs) -> anyhow::Result<()> {
    let IndexerArgs {
        manifest,
        embedded_database,
        postgres_database,
        postgres_password,
        postgres_port,
        postgres_user,
        ..
    } = args.clone();

    let args_config = args.config.clone();

    // start embedded postgres
    if embedded_database {
        info!("Starting embedded postgres");

        use fuel_indexer_lib::defaults;

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
    let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);

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
                    "✨ ✨ GraphQL Playground at: http://localhost:29987/api/playground/{}/{}", m.namespace, m.identifier
                );
            service.register_index_from_manifest(m).await?;
        }
        None => {
            service.register_indices_from_registry().await?;
            info!(
                    "✨ ✨ GraphQL Playground at: http://localhost:29987/api/playground/:namespace/:identifier"
                );
        }
    }

    let service_handle = tokio::spawn(service.run());

    #[cfg(feature = "api-server")]
    {
        let gql_handle =
            tokio::spawn(GraphQlApi::build_and_run(config.clone(), pool, tx));

        #[cfg(feature = "fuel-core-lib")]
        {
            use fuel_core::service::{Config, FuelService};
            use std::net::{IpAddr, Ipv4Addr, SocketAddr};

            if config.local_fuel_node {
                let config = Config {
                    addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000),
                    ..Config::local_node()
                };
                let node_handle = tokio::spawn(FuelService::new_node(config));

                let _ = tokio::join!(service_handle, node_handle, gql_handle);

                return Ok(());
            }
        }
    }

    #[cfg(not(feature = "api-server"))]
    {
        #[cfg(feature = "fuel-core-lib")]
        {
            use fuel_core::service::{Config, FuelService};
            use std::net::{IpAddr, Ipv4Addr, SocketAddr};

            if config.local_fuel_node {
                let config = Config {
                    addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000),
                    ..Config::local_node()
                };
                let node_handle = tokio::spawn(FuelService::new_node(config));

                let _ = tokio::join!(service_handle, node_handle);

                return Ok(());
            }
        }

        let _ = service_handle.await?;

        Ok(())
    }

    use tokio::signal::unix::{signal, Signal, SignalKind};

    let mut sighup: Signal = signal(SignalKind::hangup())?;
    let mut sigterm: Signal = signal(SignalKind::terminate())?;
    let mut sigint: Signal = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = sighup.recv() => {
            info!("Received SIGHUP. Stopping services");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM. Stopping services");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT. Stopping services");
        }
    }

    // stop embedded postgres
    if embedded_database {
        info!("Stopping embedded postgres");

        use fuel_indexer_lib::defaults;
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
