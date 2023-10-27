extern crate lazy_static;
extern crate proc_macro;

pub(crate) mod decoder;
pub(crate) mod helpers;
pub(crate) mod indexer;
pub(crate) mod native;
pub(crate) mod parse;
pub(crate) mod schema;
pub(crate) mod wasm;

use indexer::process_indexer_module;
use prometheus_metrics::process_with_prometheus_metrics
use proc_macro::TokenStream;

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn indexer(attrs: TokenStream, item: TokenStream) -> TokenStream {
    process_indexer_module(attrs, item)
}

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn metrics(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    process_with_prometheus_metrics(input)
}