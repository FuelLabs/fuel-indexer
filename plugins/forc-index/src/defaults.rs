use fuel_indexer_lib::defaults;
use std::path::PathBuf;

pub const CARGO_MANIFEST_FILE_NAME: &str = "Cargo.toml";

pub const INDEXER_LIB_FILENAME: &str = "lib.rs";
pub const CARGO_CONFIG_DIR_NAME: &str = ".cargo";
pub const CARGO_CONFIG_FILENAME: &str = "config";
pub const INDEXER_SERVICE_HOST: &str = "http://127.0.0.1:29987";
pub const WEB_API_PORT: &str = defaults::WEB_API_PORT;
pub const WASM_TARGET: &str = "wasm32-unknown-unknown";
pub const MESSAGE_PADDING: usize = 55;
pub const SUCCESS_EMOJI_PADDING: usize = 3;
pub const FAIL_EMOJI_PADDING: usize = 6;
pub const HEADER_PADDING: usize = 20;

/// Generate default cargo manifest for wasm indexer.
pub fn default_indexer_cargo_toml(indexer_name: &str) -> String {
    format!(
        r#"[package]
name = "{indexer_name}"
version = "0.0.0"
edition = "2021"
publish = false
rust-version = "1.73.0"

[lib]
crate-type = ['cdylib']

[dependencies]
anyhow = "1"
fuel-indexer-utils = {{ version = "0.21" }}
fuels = {{ version = "0.46", default-features = false }}
getrandom = {{ version = "0.2", features = ["js"] }}
serde = {{ version = "1.0", default-features = false, features = ["derive"] }}
"#
    )
}

/// Generate default cargo manifest for indexer.
pub fn default_indexer_manifest(
    namespace: &str,
    schema_filename: &str,
    indexer_name: &str,
    project_path: Option<&PathBuf>,
) -> String {
    let schema_path = match project_path {
        Some(p) => p.join("schema").join(schema_filename),
        None => {
            let p = format!("schema/{schema_filename}");
            PathBuf::from(&p)
        }
    };

    let schema_path = schema_path.display();

    format!(
        r#"# A namespace is a logical grouping of declared names. Think of the namespace
# as an organization identifier
namespace: {namespace}

# The identifier field is used to identify the given index.
identifier: {indexer_name}

# The abi option is used to provide a link to the Sway JSON ABI that is generated when you
# build your project.
abi: ~

# The particular start block after which you'd like your indexer to start indexing events.
start_block: ~

# The particular end block after which you'd like your indexer to stop indexing events.
end_block: ~

# The `fuel_client` denotes the address (host, port combination) of the running Fuel client
# that you would like your indexer to index events from. In order to use this per-indexer
# `fuel_client` option, the indexer service at which your indexer is deployed will have to run
# with the `--indexer_net_config` option.
fuel_client: ~

# The contract_id specifies which particular contract you would like your index to subscribe to.
contract_id: ~

# The graphql_schema field contains the file path that points to the GraphQL schema for the
# given index.
graphql_schema: {schema_path}

# The module field contains a file path that points to code that will be run as an executor inside
# of the indexer.
module:
  wasm: ~

# The resumable field contains a boolean that specifies whether or not the indexer should, synchronise
# with the latest block if it has fallen out of sync.
resumable: true
"#
    )
}

/// Generate default lib module for wasm indexer.
pub fn default_indexer_lib(
    indexer_name: &str,
    manifest_filename: &str,
    project_path: Option<&PathBuf>,
) -> String {
    let manifest_path = match project_path {
        Some(p) => p.join(manifest_filename),
        None => PathBuf::from(manifest_filename),
    };

    let manifest_path = manifest_path.display();

    format!(
        r#"extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "{manifest_path}")]
pub mod {indexer_name}_index_mod {{

    fn {indexer_name}_handler(block_data: BlockData) {{
        if block_data.header.height % 1000 == 0 {{
            info!("Processing Block#{{}}. (>'.')>", block_data.header.height);
        }}
        
        let block = Block::new(block_data.header.height.into(), block_data.id);
        block.save();

        for transaction in block_data.transactions.iter() {{
            let tx = Transaction::new(block_data.id, Bytes32::from(<[u8; 32]>::from(transaction.id)));
            tx.save();
        }}
    }}
}}
"#
    )
}

/// Generate default schema for indexer.
pub fn default_indexer_schema() -> String {
    r#"type Block @entity {
    id: ID!
    height: U64!
    hash: Bytes32! @unique
}

type Transaction @entity {
    id: ID!
    block: Block! @join(on:hash)
    hash: Bytes32! @unique
}

"#
    .to_string()
}

/// Generate default cargo config for indexer.
pub fn default_cargo_config() -> String {
    r#"[build]
target = "wasm32-unknown-unknown"
"#
    .to_string()
}

pub fn manifest_name(indexer_name: &str) -> String {
    format!("{indexer_name}.manifest.yaml")
}
