#![feature(test)]

extern crate test;

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
        include_str!("./../components/indices/fuel-indexer-test/fuel_indexer_test.yaml");
    pub const SIMPLE_WASM_MANIFEST: &str =
        include_str!("./../components/indices/simple-wasm/simple_wasm.yaml");
    pub const BAD_SIMPLE_WASM_MANIFEST: &str =
        include_str!("./../components/indices/simple-wasm/bad_simple_wasm.yaml");
    pub const BAD_SIMPLE_WASM_WASM: &[u8] =
        include_bytes!("./../components/indices/simple-wasm/bad_simple_wasm.wasm");
    pub const SIMPLE_WASM_WASM: &[u8] =
        include_bytes!("./../components/indices/simple-wasm/simple_wasm.wasm");
    pub const SIMPLE_WASM_SCHEMA: &str =
        include_str!("./../components/indices/simple-wasm/schema/simple_wasm.graphql");
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
        manifest.graphql_schema = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(&manifest.graphql_schema)
            .into_os_string()
            .to_str()
            .unwrap()
            .to_string();
        manifest.abi = Some(
            manifest_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join(manifest.abi.clone().unwrap())
                .into_os_string()
                .to_str()
                .unwrap()
                .to_string(),
        );
        manifest.module = Module::Wasm(
            manifest_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join(manifest.module.to_string())
                .into_os_string()
                .to_str()
                .unwrap()
                .to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    use fuel_indexer::{Executor, IndexerConfig, Manifest, WasmIndexExecutor};
    use std::{fs::File, io::Read};

    fn make_executor_metering_benchmark(b: &mut Bencher, metering_points: Option<u64>) {
        if let Ok(current_dir) = std::env::current_dir() {
            println!("Current directory: {}", current_dir.display());
        }

        if let Ok(mut current_dir) = std::env::current_dir() {
            if !current_dir.ends_with("fuel-indexer") {
                current_dir.pop();
                current_dir.pop();
            }

            if let Err(e) = std::env::set_current_dir(current_dir) {
                eprintln!("Failed to change directory: {}", e);
            }
        }

        let manifest = Manifest::from_file(
            "packages/fuel-indexer-tests/components/indices/fuel-indexer-test/fuel_indexer_test.yaml",
        )
        .unwrap();

        let wasm_bytes = match &manifest.module {
            fuel_indexer_lib::manifest::Module::Wasm(ref module) => {
                let mut bytes = Vec::<u8>::new();
                let mut file = File::open(module).unwrap();
                file.read_to_end(&mut bytes).unwrap();
                bytes
            }
            _ => panic!("unexpected"),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();

        let mut executor = rt.block_on(async {
            let mut config = IndexerConfig::default();
            config.indexer_handler_timeout = 60;

            let manifest = manifest.clone();
            let wasm_bytes = wasm_bytes.clone();

            // let pool = fuel_indexer_database::IndexerConnectionPool::connect(
            //     &config.database.to_string(),
            // )
            // .await
            // .unwrap();

            WasmIndexExecutor::new(
                &config,
                &manifest,
                wasm_bytes,
                metering_points,
            )
            .await
            .unwrap()
        });

        let blocks: Vec<fuel_indexer_types::fuel::BlockData> = vec![];
        for _ in 0..16 {
            b.iter(|| {
                rt.block_on(executor.handle_events(blocks.clone())).unwrap();
            })
        }
    }

    #[bench]
    fn executor_with_metering(b: &mut Bencher) {
        make_executor_metering_benchmark(b, Some(9_000_000_000_000));
    }

    #[bench]
    fn executor_no_metering(b: &mut Bencher) {
        make_executor_metering_benchmark(b, None);
    }
}
