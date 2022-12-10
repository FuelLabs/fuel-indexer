use quote::quote;

pub fn handler_block_native(
    handler_block: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let native_prelude = native_prelude();

    quote! {

        #native_prelude

        async fn handle_events(blocks: Vec<BlockData>) {

            #handler_block

        }
    }
}

fn native_prelude() -> proc_macro2::TokenStream {
    quote! {
        use fuel_indexer_plugin::native::*;
        use fuel_indexer_plugin::prelude::*;
        use fuel_indexer_schema::utils::{serialize, deserialize};
        use fuels_core::{abi_decoder::ABIDecoder, Parameterize, StringToken, Tokenizable};

        type B256 = [u8; 32];
    }
}
