#[cfg(not(feature = "trybuild"))]
pub mod fixtures;

use fuel_indexer_lib::config::IndexerConfigError;
use thiserror::Error;

pub const WORKSPACE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[derive(Error, Debug)]
pub enum TestError {
    #[error("Database error: {0:?}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Error retrieving DATABASE_URL environment variable: {0:?}")]
    DatabaseUrlEnvVarError(#[from] std::env::VarError),
    #[error("Error parsing DATABASE_URL environment variable: {0:?}")]
    DatabaseUrlParseError(#[from] url::ParseError),
    #[error("Error creating database pool {0:?}")]
    PoolCreationError(#[from] fuel_indexer_database::IndexerDatabaseError),
    #[error("Error tearing down database {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("IndexerConfigError: {0:?}")]
    IndexerConfigError(#[from] IndexerConfigError),
}

pub mod assets {
    use duct::cmd;
    use fuel_indexer_lib::manifest::Manifest;

    pub const FUEL_INDEXER_TEST_MANIFEST: &str =
        include_str!("./../indexers/fuel-indexer-test/fuel_indexer_test.yaml");

    pub const SIMPLE_WASM_MANIFEST: &str =
        include_str!("./../indexers/simple-wasm/simple_wasm.yaml");

    pub const SIMPLE_WASM_SCHEMA: &str =
        include_str!("./../indexers/simple-wasm/schema/simple_wasm.graphql");

    pub const SIMPLE_WASM_WASM: &[u8] =
        include_bytes!("../../../target/wasm32-unknown-unknown/release/simple_wasm.wasm");

    // NOTE: This is a hack to update the `manifest` with the proper absolute paths.
    // This is already done in the #[indexer] attribute, but since these tests test
    // modules that have already been compiled, we need to do this manually here.
    //
    // Doing this allows us to use the relative root of the the fuel-indexer/
    // repo for all test asset paths (i.e., we can simply reference all asset paths in
    // in manifest files relative from 'fuel-indexer')
    pub fn test_indexer_updated_manifest() -> Manifest {
        // TODO: Write a CI check for this
        let workspace_root = cmd!("cargo", "metadata", "--format-version=1")
            .pipe(cmd!("json_pp"))
            .pipe(cmd!("jq", ".workspace_root"))
            .read()
            .unwrap();
        let mut chars = workspace_root.chars();
        chars.next();
        chars.next_back();
        let workspace_root = chars.as_str().to_string();
        // Should mirror packages::fuel_indexer_tests::indexers::fuel_indexer_test::fuel_indexer_test.yaml
        let content = format!(
            r#"
namespace: fuel_indexer_test
fuel_client: ~
schema: {workspace_root}/packages/fuel-indexer-tests/indexers/fuel-indexer-test/schema/fuel_indexer_test.graphql
start_block: ~
end_block: ~
contract:
  abi: {workspace_root}/packages/fuel-indexer-tests/sway/test-contract1/out/debug/test-contract1-abi.json
  subscriptions:
    - fuel1jjrj8zjyjc3s4qkw345mt57mwn56lnc9zwqnt5krrx9umwxacrvs2c3jyg
identifier: index1
module:
  wasm: {workspace_root}/target/wasm32-unknown-unknown/release/fuel_indexer_test.wasm
resumable: true
predicates:
  templates:
    - name: TestPredicate1
      abi: {workspace_root}/packages/fuel-indexer-tests/sway/test-predicate1/out/debug/test-predicate1-abi.json
      id: 0xcfd60aa414972babde16215e0cb5a2739628831405a7ae81a9fc1d2ebce87145
    - name: TestPredicate2
      id: 0x1c83e1f094b47f14943066f6b6ca41ce5c3ae4e387c973e924564dac0227a896
      abi: {workspace_root}/packages/fuel-indexer-tests/sway/test-predicate2/out/debug/test-predicate2-abi.json
"#
        );
        Manifest::try_from(content.as_str()).unwrap()
    }
}

pub mod defaults {
    use std::time::Duration;

    pub const FUEL_NODE_ADDR: &str = "localhost:4000";
    pub const FUEL_NODE_HOST: &str = "localhost";
    pub const FUEL_NODE_PORT: &str = "4000";
    pub const WEB_API_ADDR: &str = "localhost:8000";
    pub const PING_CONTRACT_ID: &str =
        "68518c3ba3768c863e0d945aa18249f9516d3aa1338083ba79467aa393de109c";
    pub const TRANSFER_BASE_ASSET_ID: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    pub const SLEEP: Duration = Duration::from_secs(60 * 60 * 10);
    pub const WALLET_PASSWORD: &str = "password";
    pub const INDEXED_EVENT_WAIT: u64 = 2;
    pub const COIN_AMOUNT: u64 = 11;
    pub const MAX_BODY_SIZE: usize = 5242880; // 5MB in bytes
    pub const POSTGRES_URL: &str = "postgres://postgres:my-secret@localhost:5432";
}
