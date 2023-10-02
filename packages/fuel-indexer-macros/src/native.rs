use quote::quote;

/// Generate the handler block for the native execution environment.
pub fn handler_block_native(
    handler_block: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let native_prelude = native_prelude();

    quote! {

        #native_prelude

        async fn handle_events(blocks: Vec<BlockData>, db_conn: Arc<Mutex<Database>>) -> IndexerResult<()> {

            unsafe {
                if db.is_none() {
                    db = Some(db_conn);
                }
            }

            #handler_block

            Ok(())

        }
    }
}

/// Prelude imports for the `indexer` module.
///
/// These imports are placed below the top-level lib imports, so any
/// dependencies imported here will only be within the scope of the
/// indexer module, not within the scope of the entire lib module.
fn native_prelude() -> proc_macro2::TokenStream {
    quote! {
        static mut db: Option<Arc<Mutex<Database>>> = None;

        use fuel_indexer_utils::plugin::types::*;
        use fuel_indexer_utils::plugin::native::*;
        use fuel_indexer_utils::plugin::{serde_json, serialize, deserialize, bincode};
        use fuel_indexer_utils::plugin::serde::{Deserialize, Serialize};
        use fuels::{
            core::{codec::ABIDecoder, Configurables, traits::{Parameterize, Tokenizable}},
            types::{StringToken},
        };

    }
}

/// Generate the `main` function for the native execution module.
///
/// This should be an exact reference to `fuel_indexer::main` with a few exceptions:
///     - No references to an embedded database
///     - `--manifest` is a required option.
///     - Handlers are registered via `register_native_indexer` instead of `register_indexer_from_manifest`.
pub fn native_main() -> proc_macro2::TokenStream {
    quote! {
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

        #[tokio::main]
        async fn main() -> anyhow::Result<()> {

            let args = IndexerArgs::parse();

            let IndexerArgs { manifest, .. } = args.clone();

            let mut subsystems: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();

            subsystems.spawn(shutdown_signal_handler()?);

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

            if manifest.is_none() {
                panic!("Manifest required to use native execution.");
            }

            let p = manifest.unwrap();
            if config.verbose {
                info!("Using manifest file located at '{}'", p.display());
            }
            let manifest = Manifest::from_file(&p)?;
            service.register_native_indexer(manifest, handle_events).await?;

            subsystems.spawn(async {
                let result = service.run().await;
                if let Err(e) = result {
                    tracing::error!("Indexer Service failed: {e}");
                }
            });

            // Fuel indexer API web server always on due to feature-flagging
            //
            // TODO: https://github.com/FuelLabs/fuel-indexer/issues/1393
            //
            // #[cfg(feature = "api-server")]
            subsystems.spawn({
                let config = config.clone();
                async {
                    if let Err(e) = WebApi::build_and_run(config, pool, tx).await {
                        tracing::error!("Api Server failed: {e}");
                    }
                }
            });

            // Fuel core components removed due to feature-flagging
            //
            // TODO: https://github.com/FuelLabs/fuel-indexer/issues/1393
            //
            // #[cfg(feature = "fuel-core-lib")]
            // {
            //     use fuel_core::service::{Config, FuelService};
            //     use std::net::{IpAddr, Ipv4Addr, SocketAddr};

            //     if config.local_fuel_node {
            //         let config = Config {
            //             addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000),
            //             ..Config::local_node()
            //         };
            //         subsystems.spawn(async move {
            //             if let Err(e) = FuelService::new_node(config).await {
            //                 tracing::error!("Fuel Node failed: {e}");
            //             };
            //         });
            //     }
            // };

            // Each subsystem runs its own loop, and we require all subsystems for the
            // Indexer service to operate correctly. If any of the subsystems stops
            // running, the entire Indexer Service exits.
            if subsystems.join_next().await.is_some() {
                subsystems.shutdown().await;
            }

            Ok(())
        }
    }
}
