extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn metrics(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemFn);
    let fn_name = &ast.sig.ident;
    let label = fn_name.to_string();
    let fn_inputs = &ast.sig.inputs;
    let fn_output = &ast.sig.output;
    let fn_vis = &ast.vis;
    let block = &ast.block;

    let (asyncness, awaitness) = if ast.sig.asyncness.is_none() {
        (quote! {}, quote! {})
    } else {
        (quote! {async}, quote! {.await})
    };

    let input_idents = fn_inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Typed(typed) => typed.pat.clone(),
            syn::FnArg::Receiver(_) => panic!("`self` arguments are not supported"),
        })
        .collect::<Vec<_>>();

    let gen = quote! {
        #fn_vis #asyncness fn #fn_name(#fn_inputs) #fn_output {
            let result = {
                let start_time = Instant::now();
                #asyncness fn inner(#fn_inputs) #fn_output #block
                let res = inner(#(#input_idents),*)#awaitness;

                METRICS
                    .db
                    .postgres
                    .record(#label, start_time.elapsed().as_millis() as f64);
                res
            };
            result
        }
    };

    gen.into()
}
