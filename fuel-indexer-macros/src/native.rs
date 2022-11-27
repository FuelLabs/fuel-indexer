use quote::quote;

pub fn handler_block_native(
    handler_block: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let native_prelude = native_prelude();

    quote! {
        #native_prelude

        #[no_mangle]
        pub extern "C" fn handle_events(blocks: Vec<BlockData>) {

            let filter = match std::env::var_os("RUST_LOG") {
                Some(_) => {
                    EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
                }
                None => EnvFilter::new("info"),
            };

            if let Err(e) = tracing_subscriber::fmt::Subscriber::builder()
                .with_writer(std::io::stderr)
                .with_env_filter(filter)
                .try_init() {
                debug!("Could not init tracing subscriber: {}", e);
            }

            #handler_block
        }
    }
}

fn native_prelude() -> proc_macro2::TokenStream {
    quote! {
        use fuel_indexer_plugin::{
            NativeEntity, NativeLogger, debug, tracing_subscriber,
            tracing_subscriber::filter::EnvFilter
        };
    }
}
