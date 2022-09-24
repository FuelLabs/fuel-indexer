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
        std::env::set_var("COMPILE_TEST_PREFIX", env!("CARGO_MANIFEST_DIR"));

        t.compile_fail("./../../tests/assets/macro-data/fail_if_attribute_args_include_self.rs");
        t.compile_fail("./../../tests/assets/macro-data/fail_if_attribute_args_not_included.rs");
        t.compile_fail(
            "./../../tests/assets/macro-data/fail_if_all_attribute_args_not_included.rs",
        );
        t.pass("./../../tests/assets/macro-data/pass_if_indexer_is_valid_single_type.rs");
        t.pass("./../../tests/assets/macro-data/pass_if_indexer_is_valid_multi_type.rs");
        t.compile_fail(
            "./../../tests/assets/macro-data/fail_if_attribute_schema_arg_is_invalid.rs",
        );
        t.compile_fail(
            "./../../tests/assets/macro-data/fail_if_attribute_abi_arg_includes_invalid_type.rs",
        );
        t.compile_fail("./../../tests/assets/macro-data/fail_if_indexer_module_is_empty.rs");
    }
}
