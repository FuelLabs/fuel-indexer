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
                db = Some(db_conn);
            }

            #handler_block

            Ok(())

        }
    }
}

/// Prelude imports for the _indexer_ module.
///
/// These imports are placed below the top-level lib imports, so any
/// dependencies imported here will only be within the scope of the
/// indexer module, not within the scope of the entire lib module.
fn native_prelude() -> proc_macro2::TokenStream {
    quote! {
        type B256 = [u8; 32];

        static mut db: Option<Arc<Mutex<Database>>> = None;

        use fuel_indexer_utils::{
            fuels::{
                core::abi_decoder::ABIDecoder,
                macros::{Parameterize, Tokenizable},
                types::{
                    traits::{Parameterize, Tokenizable},
                    StringToken,
                },
            },
            native::*,
            prelude::{
                bincode, deserialize,
                serde_json, serialize,
            },
            serde::{Deserialize, Serialize},
            types::*,
        };
    }
}
