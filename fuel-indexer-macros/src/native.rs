use quote::quote;

pub fn handler_block_native(
    handler_block: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let native_prelude = native_prelude();

    quote! {
        #native_prelude

        #[no_mangle]
        pub extern "C" fn handle_events(blocks: Vec<BlockData>) {

            let _ = NativeLogger::init();

            #handler_block
        }
    }
}

fn native_prelude() -> proc_macro2::TokenStream {
    quote! {
        use std::collections::HashMap;

    }
}
