pub mod fixtures;

pub const WORKSPACE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

pub mod assets {
    pub const FUEL_INDEXER_TEST_MANIFEST: &str =
        include_str!("./../assets/fuel_indexer_test.yaml");
    pub const SIMPLE_WASM_MANIFEST: &str = include_str!("./../assets/simple_wasm.yaml");
    pub const BAD_SIMPLE_WASM_MANIFEST: &str =
        include_str!("./../assets/bad_simple_wasm.yaml");
    pub const BAD_SIMPLE_WASM_WASM: &[u8] =
        include_bytes!("./../assets/bad_simple_wasm.wasm");
    pub const SIMPLE_WASM_WASM: &[u8] = include_bytes!("./../assets/simple_wasm.wasm");
    pub const SIMPLE_WASM_SCHEMA: &str = include_str!("./../assets/simple_wasm.graphql");
}

pub mod defaults {
    use std::time::Duration;

    pub const FUEL_NODE_ADDR: &str = "127.0.0.1:4000";
    pub const FUEL_NODE_HOST: &str = "127.0.0.1";
    pub const FUEL_NODE_PORT: &str = "4000";
    pub const WEB_API_ADDR: &str = "127.0.0.1:8000";
    pub const PING_CONTRACT_ID: &str =
        "68518c3ba3768c863e0d945aa18249f9516d3aa1338083ba79467aa393de109c";
    pub const TRANSFER_BASE_ASSET_ID: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    pub const SLEEP: Duration = Duration::from_secs(60 * 60 * 10);
    pub const WALLET_PASSWORD: &str = "password";
    pub const INDEXED_EVENT_WAIT: u64 = 2;
    pub const COIN_AMOUNT: u64 = 11;

    // Note: We need the contract ID in order to connect an instance to
    // the node provider for E2E tests, but spawning a task that is supposed
    // to return the contract ID won't give us the ID until the task is completed.
    pub const CURRENT_TEST_CONTRACT_ID_STR: &str =
        "fuel1u47xjlucyjf2hkn874675fgdfue0vmyne72ucch74jcgu457rluq236j5e";
    pub const MAX_BODY: usize = 1024 * 1024 * 5; // 5MB in bytes
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
                .join(manifest.module.path())
                .into_os_string()
                .to_str()
                .unwrap()
                .to_string(),
        );
    }
}
