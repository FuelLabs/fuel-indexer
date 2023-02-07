use fuel_indexer_lib::defaults;

pub const CARGO_MANIFEST_FILE_NAME: &str = "Cargo.toml";
pub const INDEX_LIB_FILENAME: &str = "lib.rs";
pub const CARGO_CONFIG_DIR_NAME: &str = ".cargo";
pub const CARGO_CONFIG_FILENAME: &str = "config";
pub const INDEXER_SERVICE_HOST: &str = "http://127.0.0.1:29987";
pub const GRAPHQL_API_PORT: &str = defaults::GRAPHQL_API_PORT;
pub const INDEX_TARGET: &str = "wasm32-unknown-unknown";

pub fn default_native_index_cargo_toml(index_name: &str) -> String {
    format!(
        r#"[package]
name = "{index_name}"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ['cdylib']

[dependencies]
fuel-indexer-macros = {{ version = "0.2", default-features = false }}
fuel-indexer-plugin = {{ version = "0.2", features = ["native-execution"] }}
fuel-indexer-schema = {{ version = "0.2", default-features = false }}
fuel-tx = "0.23"
fuels = {{ version = "0.30", features = ["fuel-core-lib"] }}
fuels-core = "0.30"
fuels-types = "0.30"
getrandom = {{ version = "0.2", features = ["js"] }}
serde = {{ version = "1.0", default-features = false, features = ["derive"] }}
"#
    )
}

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
fuel-indexer-macros = {{ version = "0.2", default-features = false }}
fuel-indexer-plugin = {{ version = "0.2" }}
fuel-indexer-schema = {{ version = "0.2", default-features = false }}
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
        r#"# A namespace is a logical grouping of declared names. Think of the namespace
# as an organization identifier
namespace: {namespace}

# The identifier field is used to identify the given index.
identifier: {index_name}

# The abi option is used to provide a link to the Sway JSON ABI that is generated when you
# build your project.
abi: ~

# The particular start block after which you'd like your indexer to start indexing events.
start_block: ~

# The contract_id specifies which particular contract you would like your index to subscribe to.
contract_id: ~

# The graphql_schema field contains the file path that points to the GraphQL schema for the
# given index.
graphql_schema: {project_path}/schema/{index_name}.schema.graphql

# The module field contains a file path that points to code that will be run as an executor inside
# of the indexer.
# Important: At this time, wasm is the preferred method of execution.
module:
    wasm: ~

# The report_metrics field contains boolean whether or not to report Prometheus  metrics to the
# Fuel backend
report_metrics: true"#
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

        let block_id = first8_bytes_to_u64(block_data.id);
        let block = Block{{ id: block_id, height: block_data.height, hash: block_data.id }};
        block.save();

        for transaction in block_data.transactions.iter() {{
            Logger::info("Handling a transaction (>'.')>");

            let tx = Tx{{ id: first8_bytes_to_u64(transaction.id), block: block_id, hash: transaction.id }};
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
    block: Block
    tx: Tx
}

type Block {
    id: ID!
    height: UInt8!
    hash: Bytes32! @unique
}

type Tx {
    id: ID!
    block: Block!
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

pub fn manifest_name(index_name: &str) -> String {
    format!("{index_name}.manifest.yaml")
}
