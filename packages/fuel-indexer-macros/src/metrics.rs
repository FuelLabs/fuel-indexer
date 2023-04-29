use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

pub fn process_with_prometheus_metrics(
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let ast = parse_macro_input!(input as ItemFn);
    let label = parse_macro_input!(attrs as Ident).to_string();
    let fn_name = &ast.sig.ident;
    let fn_inputs = &ast.sig.inputs;
    let fn_output = &ast.sig.output;
    let block = &ast.block;

    let (asyncness, awaitness) = if ast.sig.asyncness.is_none() {
        (quote! {}, quote! {})
    } else {
        (quote! {async}, quote! {await})
    };

    let input_idents = fn_inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Typed(typed) => typed.pat.clone(),
            syn::FnArg::Receiver(_) => panic!("`self` arguments are not supported"),
        })
        .collect::<Vec<_>>();

    let gen = quote! {
        #asyncness fn #fn_name(#fn_inputs) #fn_output {
            println!(">> JUST WRAPPED {label} <<<");
            let result = || #block(#(#input_idents),*)#awaitness;
            result
        }
    };

    gen.into()
}
