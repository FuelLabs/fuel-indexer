extern crate proc_macro;
use proc_macro::TokenStream;

mod indexer;
mod native;
mod parse;
mod schema;
mod wasm;
use indexer::{process_block_attribute_fn, process_indexer_module};

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn indexer(attrs: TokenStream, item: TokenStream) -> TokenStream {
    process_indexer_module(attrs, item)
}

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn block(attrs: TokenStream, item: TokenStream) -> TokenStream {
    process_block_attribute_fn(attrs, item)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros() {
        let t = trybuild::TestCases::new();
        std::env::set_var("COMPILE_TEST_PREFIX", env!("CARGO_MANIFEST_DIR"));

        t.compile_fail("test_data/fail_self.rs");
        t.compile_fail("test_data/fail_args.rs");
        t.compile_fail("test_data/fail_args2.rs");
        t.pass("test_data/success.rs");
        t.pass("test_data/success2.rs");
        t.compile_fail("test_data/fail_badschema.rs");
        t.compile_fail("test_data/fail_unknown_type.rs");
        t.compile_fail("test_data/fail_empty.rs");
    }
}
