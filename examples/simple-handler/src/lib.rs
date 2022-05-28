mod schema;

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn graphql_schema(inputs: TokenStream) -> TokenStream {
    schema::process_graphql_schema(inputs)
}
