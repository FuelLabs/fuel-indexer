use quote::quote;
//Blocks(Vec<BlockData>),
//GetObject(Vec<u8>),
//PutObject(Vec<u8>),
//Commit

pub fn handler_block_native(handler_block: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let native_prelude = native_prelude();

    quote! {
        #native_prelude

        fn handle_events(blocks: Vec<BlockData>) {
            #handler_block
        }
    }
}

fn native_prelude() -> proc_macro2::TokenStream {
    quote! {
        todo!("Native prelude here")
    }
}
