use quote::quote;

/// Generate the handler block for the wasm execution environment.
pub fn handler_block_wasm(
    handler_block: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let wasm_prelude = wasm_prelude();

    quote! {

        #wasm_prelude

        #[no_mangle]
        fn handle_events(blob: *mut u8, len: usize) {
            use fuel_indexer_utils::plugin::deserialize;
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

/// Prelude imports for the _indexer_ module.
///
/// These imports are placed below the top-level lib imports, so any
/// dependencies imported here will only be within the scope of the
/// indexer module, not within the scope of the entire lib module.
fn wasm_prelude() -> proc_macro2::TokenStream {
    quote! {
        use alloc::{format, vec, vec::Vec};
        use std::str::FromStr;

        type B256 = [u8; 32];

        use fuel_indexer_utils::plugin::types::*;
        use fuel_indexer_utils::plugin::wasm::*;
        use fuel_indexer_utils::plugin::{serde_json, serialize, deserialize, bincode};
        use fuel_indexer_utils::plugin::serde::{Deserialize, Serialize};
        use fuels::{
            core::{codec::ABIDecoder, Configurables, traits::{Parameterize, Tokenizable}},
            types::{StringToken, param_types::ParamType},
        };
    }
}
