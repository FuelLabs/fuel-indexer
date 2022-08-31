extern crate proc_macro;
use proc_macro::TokenStream;

mod indexer;
mod native;
mod parse;
mod schema;
mod wasm;
use indexer::process_indexer_module;

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn indexer(attrs: TokenStream, item: TokenStream) -> TokenStream {
    process_indexer_module(attrs, item)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros() {
        let t = trybuild::TestCases::new();

        t.compile_fail("test_data/fail_self.rs");
        //t.compile_fail("test_data/fail_args.rs");
        //t.pass("test_data/success.rs");
        //t.compile_fail("test_data/fail_noschema.rs");
        //t.compile_fail("test_data/fail_badschema.rs");
    }
}
