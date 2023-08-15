pub mod fixtures;

pub const WORKSPACE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

use fuel_indexer_lib::config::IndexerConfigError;
use thiserror::Error;

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
    pub const FUEL_INDEXER_TEST_MANIFEST: &str =
        include_str!("./../indexers/fuel-indexer-test/fuel_indexer_test.yaml");
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

pub mod utils {

    use super::WORKSPACE_ROOT;
    use fuel_indexer_lib::manifest::{Manifest, Module};
    use std::path::Path;

    // NOTE: This is a hack to update the `manifest` with the proper absolute paths.
    // This is already done in the #[indexer] attribute, but since these tests test
    // modules that have already been compiled, we need to do this manually here.
    //
    // Doing this allows us to use the relative root of the the fuel-indexer/
    // repo for all test asset paths (i.e., we can simply reference all asset paths in
    // in manifest files relative from 'fuel-indexer')
    pub fn update_test_manifest_asset_paths(manifest: &mut Manifest) {
        let manifest_dir = Path::new(WORKSPACE_ROOT);
        let graphql_schema = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(manifest.graphql_schema())
            .into_os_string()
            .to_str()
            .unwrap()
            .to_string();

        manifest.set_graphql_schema(graphql_schema);

        let abi = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(manifest.abi().clone().unwrap())
            .into_os_string()
            .to_str()
            .unwrap()
            .to_string();

        manifest.set_abi(abi);

        let module = Module::Wasm(
            manifest_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join(manifest.module().to_string())
                .into_os_string()
                .to_str()
                .unwrap()
                .to_string(),
        );

        manifest.set_module(module);
    }
}
