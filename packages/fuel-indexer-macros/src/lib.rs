extern crate lazy_static;
extern crate proc_macro;

pub(crate) mod constant;
pub(crate) mod helpers;
pub(crate) mod indexer;
pub(crate) mod native;
pub(crate) mod parse;
pub(crate) mod schema;
pub(crate) mod wasm;

use indexer::process_indexer_module;
use proc_macro::TokenStream;

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn indexer(attrs: TokenStream, item: TokenStream) -> TokenStream {
    process_indexer_module(attrs, item)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_success_and_failure_macros() {
        let t = trybuild::TestCases::new();
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        std::env::set_var("COMPILE_TEST_PREFIX", manifest_dir);

        let macro_data_root = std::path::Path::new(manifest_dir)
            .join("..")
            .join("fuel-indexer-tests")
            .join("trybuild");

        t.compile_fail(macro_data_root.join("fail_if_attribute_args_include_self.rs"));
        t.compile_fail(macro_data_root.join("fail_if_attribute_args_not_included.rs"));
        t.pass(macro_data_root.join("pass_if_indexer_is_valid_single_type.rs"));
        t.pass(macro_data_root.join("pass_if_indexer_is_valid_multi_type.rs"));
        t.compile_fail(
            macro_data_root.join("fail_if_attribute_schema_arg_is_invalid.rs"),
        );
        t.compile_fail(
            macro_data_root.join("fail_if_attribute_abi_arg_includes_invalid_type.rs"),
        );
        t.compile_fail(macro_data_root.join("fail_if_indexer_module_is_empty.rs"));
    }
}
