use quote::quote;

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

fn native_prelude() -> proc_macro2::TokenStream {
    quote! {
        use fuel_indexer_plugin::native::*;
        use fuel_indexer_plugin::prelude::*;
        use fuel_indexer_schema::utils::{serialize, deserialize};
        use fuels_core::abi_decoder::ABIDecoder;
        use fuels_types::StringToken;
        use fuels_types::traits::{Parameterize, Tokenizable};
        use fuels_macros::{Parameterize, Tokenizable};

        type B256 = [u8; 32];

        static mut db: Option<Arc<Mutex<Database>>> = None;

    }
}
