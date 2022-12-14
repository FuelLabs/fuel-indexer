use fuel_indexer_lib::defaults;

pub const CARGO_MANIFEST_FILE_NAME: &str = "Cargo.toml";
pub const INDEX_LIB_FILENAME: &str = "lib.rs";
pub const DEFAULT_NAMESPACE: &str = "fuel";
pub const CARGO_CONFIG_DIR_NAME: &str = ".cargo";
pub const CARGO_CONFIG_FILENAME: &str = "config";
pub const DEFAULT_INDEXER_URL: &str = "http://127.0.0.1:29987";
pub const DEFAULT_GRAPHQ_API_PORT: &str = defaults::GRAPHQL_API_PORT;

pub fn default_index_cargo_toml(index_name: &str) -> String {
    format!(
        r#"[package]
name = "{index_name}"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ['cdylib']

[dependencies]
fuel-indexer-macros = {{ version = "0.1", default-features = false }}
fuel-indexer-plugin = {{ version = "0.1" }}
fuel-indexer-schema = {{ version = "0.1", default-features = false }}
fuel-tx = "0.23"
fuels-core = "0.30"
fuels-types = "0.30"
getrandom = {{ version = "0.2", features = ["js"] }}
serde = {{ version = "1.0", default-features = false, features = ["derive"] }}
"#
    )
}

pub fn default_index_manifest(
    namespace: &str,
    index_name: &str,
    project_path: &str,
) -> String {
    format!(
        r#"namespace: {namespace}
identifier: {index_name}
# abi: /path/to/your/contract-abi.json
graphql_schema: {project_path}/schema/{index_name}.schema.graphql
module:
  wasm: /path/to/your/index_wasm_module.wasm
"#
    )
}

pub fn default_index_lib(
    index_name: &str,
    manifest_filename: &str,
    path: &str,
) -> String {
    format!(
        r#"extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(manifest = "{path}/{manifest_filename}")]
pub mod {index_name}_index_mod {{

    fn {index_name}_handler(block_data: BlockData) {{
        Logger::info("Processing a block. (>'.')>");

        let block = Block{{ id: u64_id(block.id), height: block.height }};
        block.save();

        for transaction in block.transactions.iter() {{
            Logger::info("Handling a transaction (>'.')>");

            let tx = Tx{{ id: u64_id(tx.id), block: block.id }};
            tx.save();
        }}
    }}
}}
"#
    )
}

pub fn default_index_schema() -> String {
    r#"schema {
    query: QueryRoot
}

type QueryRoot {
    account: Account
}

type Block {
    id: ID!
    height: UInt8!
}

type Tx {
    id: ID!
    block: Block!
}

"#
    .to_string()
}

pub fn default_cargo_config() -> String {
    r#"[build]
target = "wasm32-unknown-unknown"
"#
    .to_string()
}
