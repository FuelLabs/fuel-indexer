use fuel_indexer_lib::defaults;
use std::path::PathBuf;

pub const CARGO_MANIFEST_FILE_NAME: &str = "Cargo.toml";

pub const INDEXER_LIB_FILENAME: &str = "lib.rs";
pub const INDEXER_BINARY_FILENAME: &str = "main.rs";
pub const CARGO_CONFIG_DIR_NAME: &str = ".cargo";
pub const CARGO_CONFIG_FILENAME: &str = "config";
pub const INDEXER_SERVICE_HOST: &str = "http://127.0.0.1:29987";
// pub const WEB_API_HOST: &str = defaults::WEB_API_HOST;
pub const WEB_API_PORT: &str = defaults::WEB_API_PORT;
pub const WASM_TARGET: &str = "wasm32-unknown-unknown";
pub const INDEXER_TARGET: &str = "wasm32-unknown-unknown";
pub const BUILD_RELEASE_PROFILE: &str = "true";
pub const MESSAGE_PADDING: usize = 55;
pub const SUCCESS_EMOJI_PADDING: usize = 3;
pub const FAIL_EMOJI_PADDING: usize = 6;
pub const HEADER_PADDING: usize = 20;

pub fn default_native_indexer_cargo_toml(indexer_name: &str) -> String {
    format!(
        r#"[package]
name = "{indexer_name}"
version = "0.0.0"
edition = "2021"
publish = false

[[bin]]
name = "{indexer_name}"
path = "src/main.rs"

[dependencies]
async-trait = {{ version = "0.1" }}
fuel-indexer = {{ version = "0.18", default-features = false }}
fuel-indexer-utils = {{ version = "0.18", features = ["native-execution"] }}
fuels = {{ version = "0.43", default-features = false, features = ["std"] }}
serde = {{ version = "1.0", default-features = false, features = ["derive"] }}
"#
    )
}

pub fn default_indexer_cargo_toml(indexer_name: &str) -> String {
    format!(
        r#"[package]
name = "{indexer_name}"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ['cdylib']

[dependencies]
fuel-indexer-utils = {{ version = "0.18" }}
fuels = {{ version = "0.43", default-features = false }}
serde = {{ version = "1.0", default-features = false, features = ["derive"] }}
"#
    )
}

pub fn default_indexer_manifest(
    namespace: &str,
    schema_filename: &str,
    indexer_name: &str,
    project_path: Option<&PathBuf>,
    is_native: bool,
) -> String {
    let schema_path = match project_path {
        Some(p) => p.join("schema").join(schema_filename),
        None => {
            let p = format!("schema/{schema_filename}");
            PathBuf::from(&p)
        }
    };

    let module = if is_native {
        r#"
    native: ~"#
    } else {
        r#"
    wasm: ~"#
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
# Important: At this time, wasm is the preferred method of execution.
module: {module}

# The report_metrics field contains boolean whether or not to report Prometheus  metrics to the
# Fuel backend
report_metrics: true

# The resumable field contains a boolean that specifies whether or not the indexer should, synchronise
# with the latest block if it has fallen out of sync.
resumable: true
"#
    )
}

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
        info!("Processing Block#{{}}. (>'.')>", block_data.height);

        let block_id = id8(block_data.id);
        let block = Block{{ id: block_id, height: block_data.height, hash: block_data.id }};
        block.save();

        for transaction in block_data.transactions.iter() {{
            let tx = Transaction{{ id: id8(transaction.id), block: block_data.id, hash: Bytes32::from(<[u8; 32]>::from(transaction.id)) }};
            tx.save();
        }}
    }}
}}
"#
    )
}

pub fn default_indexer_binary(
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

    async fn {indexer_name}_handler(block_data: BlockData) {{
        info!("Processing Block#{{}}. (>'.')>", block_data.height);

        let block_id = id8(block_data.id);
        let block = Block{{ id: block_id, height: block_data.height, hash: block_data.id }};
        block.save().await;

        for transaction in block_data.transactions.iter() {{
            let tx = Transaction{{ id: id8(transaction.id), block: block_data.id, hash: Bytes32::from(<[u8; 32]>::from(transaction.id)) }};
            tx.save().await;
        }}
    }}
}}
"#
    )
}

pub fn default_indexer_schema() -> String {
    r#"type Block {
    id: ID!
    height: UInt8!
    hash: Bytes32! @unique
}

type Transaction {
    id: ID!
    block: Block! @join(on:hash)
    hash: Bytes32! @unique
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

pub fn manifest_name(indexer_name: &str) -> String {
    format!("{indexer_name}.manifest.yaml")
}
