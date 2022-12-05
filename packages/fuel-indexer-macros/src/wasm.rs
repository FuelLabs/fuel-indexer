use quote::quote;

pub fn handler_block_wasm(
    handler_block: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let wasm_prelude = wasm_prelude();

    quote! {

        #wasm_prelude

        #[no_mangle]
        fn handle_events(blob: *mut u8, len: usize) {
            use fuel_indexer_schema::utils::deserialize;
            let bytes = unsafe { Vec::from_raw_parts(blob, len, len) };
            let blocks: Vec<BlockData> = match deserialize(&bytes) {
                Ok(blocks) => blocks,
                Err(msg) => {
                    // TODO: probably need some error codes to send back to runtime.
                    core::mem::forget(bytes);
                    Logger::error(&msg);
                    return;
                }
            };
            core::mem::forget(bytes);

            #handler_block
        }
    }
}

fn wasm_prelude() -> proc_macro2::TokenStream {
    quote! {
        use alloc::{format, vec, vec::Vec};
        use fuel_indexer_plugin::wasm::*;
        use fuel_indexer_plugin::prelude::*;
        use fuel_indexer_schema::utils::{serialize, deserialize};
        use fuels_core::{abi_decoder::ABIDecoder, Parameterize, StringToken, Tokenizable};
        use std::collections::HashMap;

        type B256 = [u8; 32];
    }
}
