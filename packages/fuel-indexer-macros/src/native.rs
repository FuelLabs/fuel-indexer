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
        extern crate alloc;

        type B256 = [u8; 32];

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
pub fn native_main() -> proc_macro2::TokenStream {
    quote! {
        #[tokio::main]
        async fn main() -> anyhow::Result<()> {

            let args = IndexerArgs::parse();

            let IndexerArgs { manifest, .. } = args.clone();


            let config = args
            .config
            .as_ref()
            .map(IndexerConfig::from_file)
            .unwrap_or(Ok(IndexerConfig::from(args)))?;

            init_logging(&config).await?;

            info!("Configuration: {:?}", config);

            let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);

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

            let service_handle = tokio::spawn(service.run());
            let web_handle = tokio::spawn(WebApi::build_and_run(config.clone(), pool, tx));

            let _ = tokio::join!(service_handle, web_handle);

            Ok(())
        }
    }
}
