pub mod assets {
    pub const MANIFEST: &str = include_str!("./../assets/simple_wasm.yaml");
    pub const BAD_MANIFEST: &str = include_str!("./../assets/bad_simple_wasm.yaml");
    pub const BAD_WASM_BYTES: &[u8] = include_bytes!("./../assets/bad_simple_wasm.wasm");
    pub const WASM_BYTES: &[u8] = include_bytes!("./../assets/simple_wasm.wasm");
}

pub mod utils {
    use fuels::prelude::TxParameters;

    pub fn tx_params() -> TxParameters {
        let gas_price = 0;
        let gas_limit = 1_000_000;
        let byte_price = 0;
        TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
    }
}

pub mod defaults {
    use std::time::Duration;

    pub const FUEL_NODE_ADDR: &str = "127.0.0.1:4000";
    pub const FUEL_NODE_HOST: &str = "127.0.0.1";
    pub const FUEL_NODE_PORT: &str = "4000";
    pub const WEB_API_ADDR: &str = "127.0.0.1:8000";
    pub const PING_CONTRACT_ID: &str =
        "68518c3ba3768c863e0d945aa18249f9516d3aa1338083ba79467aa393de109c";
    pub const SLEEP: Duration = Duration::from_secs(60 * 60 * 10);
    pub const WALLET_PASSWORD: &str = "password";
}

pub mod fixtures {
    use super::defaults;
    use super::utils::tx_params;
    use fuel_indexer_database::IndexerConnectionPool;
    use fuels::{
        prelude::{
            setup_single_asset_coins, setup_test_client, AssetId, Config, Contract,
            Provider, WalletUnlocked, DEFAULT_COIN_AMOUNT,
        },
        signers::Signer,
    };
    use fuels_abigen_macro::abigen;
    use fuels_core::parameters::StorageConfiguration;
    use sqlx::{pool::Pool, Postgres};
    use std::path::Path;

    abigen!(
        FuelIndexer,
        "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
    );

    const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

    pub async fn postgres_connection(db_url: &str) -> Pool<Postgres> {
        match IndexerConnectionPool::connect(db_url).await.unwrap() {
            IndexerConnectionPool::Postgres(p) => p,
            _ => panic!("Should be postgres!"),
        }
    }

    pub fn http_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    pub async fn setup_fuel_client() {
        let workspace_dir = Path::new(WORKSPACE_DIR);

        let wallet_path = workspace_dir.join("assets//wallet.json");
        let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

        let mut wallet = WalletUnlocked::load_keystore(
            &wallet_path_str,
            defaults::WALLET_PASSWORD,
            None,
        ).unwrap();

        let bin_path = workspace_dir
            .join("contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin");
        let bin_path_str = bin_path.as_os_str().to_str().unwrap();
        let _compiled = Contract::load_sway_contract(bin_path_str, &None).unwrap();

        let number_of_coins = 11;
        let asset_id = AssetId::zeroed();
        let coins = setup_single_asset_coins(
            wallet.address(),
            asset_id,
            number_of_coins,
            DEFAULT_COIN_AMOUNT,
        );

        let config = Config {
            utxo_validation: false,
            addr: defaults::FUEL_NODE_ADDR.parse().unwrap(),
            ..Config::local_node()
        };

        let (client, _) = setup_test_client(coins, vec![], Some(config), None).await;

        let provider = Provider::new(client);

        wallet.set_provider(provider.clone());

        let contract_id = Contract::deploy(
            &bin_path_str,
            &wallet,
            tx_params(),
            StorageConfiguration::default(),
        )
        .await
        .unwrap();
        let contract = FuelIndexerBuilder::new(contract_id.to_string(), wallet);
    }
}
