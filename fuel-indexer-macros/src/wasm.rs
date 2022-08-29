use quote::quote;

pub fn handler_block_wasm(handler_block: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        #[no_mangle]
        fn handle_events(blob: *mut u8, len: usize) {
            use fuel_indexer_schema::deserialize;
            let bytes = unsafe { Vec::from_raw_parts(blob, len, len) };
            let blocks: Vec<BlockData> = match deserialize(&bytes) {
                Ok(blocks) => blocks,
                Err(msg) => {
                    // TODO: probably need some error codes to send back to runtime.
                    Logger::error(&msg);
                    return;
                }
            };
            core::mem::forget(bytes);

            #handler_block

        }
    }
}

