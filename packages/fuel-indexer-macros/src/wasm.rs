use quote::quote;

/// Generate the handler block for the wasm execution environment.
pub fn handler_block_wasm(
    handler_block: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let wasm_prelude = wasm_prelude();

    let panic_hook = panic_hook();

    quote! {

        #wasm_prelude

        #panic_hook

        #[no_mangle]
        fn handle_events(blob: *mut u8, len: usize) {


            if let Err(e) = handle_events_internal(blob, len) {
                unsafe {
                    ERROR_MESSAGE = format!("{e:?}");
                    early_exit(WasmIndexerError::GeneralError);
                }
            }
        }

        #[no_mangle]
        fn handle_events_internal(blob: *mut u8, len: usize) -> anyhow::Result<()> {
            register_panic_hook();

            use fuel_indexer_utils::plugin::deserialize;
            let bytes = unsafe { Vec::from_raw_parts(blob, len, len) };
            let blocks: Vec<BlockData> = match deserialize(&bytes) {
                Ok(blocks) => blocks,
                Err(msg) => {
                    core::mem::forget(bytes);
                    Logger::error(&msg);
                    early_exit(WasmIndexerError::DeserializationError)
                }
            };
            core::mem::forget(bytes);

            #handler_block

            Ok(())
        }
    }
}

/// Panic hook for the indexer.
///
/// When a panic occurs, the message is stored in a `static mut` `String` and a
/// `WasmIndexerError::Panic` error code is returned. The message is then
/// retrieved by the indexer service and logged.
fn panic_hook() -> proc_macro2::TokenStream {
    quote! {
        static mut ERROR_MESSAGE: String = String::new();

        #[no_mangle]
        fn get_error_message_ptr() -> *const u8 {
            unsafe { ERROR_MESSAGE.as_ptr() }
        }

        #[no_mangle]
        fn get_error_message_len() -> u32 {
            unsafe { ERROR_MESSAGE.len() as u32 }
        }

        #[no_mangle]
        fn register_panic_hook() {
            use std::panic;
            use std::sync::Once;
            static SET_HOOK: Once = Once::new();

            SET_HOOK.call_once(|| {
                panic::set_hook(Box::new(|info| {
                    unsafe {
                        ERROR_MESSAGE = info.to_string();
                    }
                    early_exit(WasmIndexerError::Panic);
                }));
            });
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
        use anyhow::Context;
        use alloc::{format, vec, vec::Vec};
        use std::str::FromStr;

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
