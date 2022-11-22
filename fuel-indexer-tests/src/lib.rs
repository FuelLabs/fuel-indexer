pub mod assets {
    pub const MANIFEST: &str = include_str!("./../assets/simple_wasm.yaml");
    pub const BAD_MANIFEST: &str = include_str!("./../assets/bad_simple_wasm.yaml");
    pub const BAD_WASM_BYTES: &[u8] = include_bytes!("./../assets/bad_simple_wasm.wasm");
    pub const WASM_BYTES: &[u8] = include_bytes!("./../assets/simple_wasm.wasm");
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
    pub const INDEXED_EVENT_WAIT: u64 = 5;
    pub const COIN_AMOUNT: u64 = 11;
}

pub mod fixtures {

    use crate::defaults;
    use fuel_indexer_database::IndexerConnectionPool;
    use fuels::prelude::TxParameters;
    use fuels::{
        prelude::{
            setup_single_asset_coins, setup_test_client, AssetId, Bech32ContractId,
            Config, Contract, Provider, WalletUnlocked, DEFAULT_COIN_AMOUNT,
        },
        signers::Signer,
    };
    use fuels_core::parameters::StorageConfiguration;
    use sqlx::{pool::Pool, Postgres};
    use tracing::info;
    use tracing_subscriber::filter::EnvFilter;

    pub async fn postgres_connection(db_url: &str) -> Pool<Postgres> {
        match IndexerConnectionPool::connect(db_url).await.unwrap() {
            IndexerConnectionPool::Postgres(p) => p,
            _ => panic!("Should be postgres."),
        }
    }

    pub fn http_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    pub fn tx_params() -> TxParameters {
        let gas_price = 0;
        let gas_limit = 1_000_000;
        let byte_price = 0;
        TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
    }

    pub async fn setup_test_fuel_node(
        wallet_path: &str,
        contract_bin_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let filter = match std::env::var_os("RUST_LOG") {
            Some(_) => {
                EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
            }
            None => EnvFilter::new("info"),
        };

        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .try_init();

        let mut wallet =
            WalletUnlocked::load_keystore(wallet_path, defaults::WALLET_PASSWORD, None)
                .unwrap();

        let _compiled = Contract::load_contract(contract_bin_path, &None).unwrap();

        let number_of_coins = defaults::COIN_AMOUNT;
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

        let (client, _) =
            setup_test_client(coins, vec![], Some(config), None, None).await;

        let provider = Provider::new(client);

        wallet.set_provider(provider.clone());

        let contract_id = Contract::deploy(
            contract_bin_path,
            &wallet,
            tx_params(),
            StorageConfiguration::default(),
        )
        .await
        .unwrap();

        let contract_id = contract_id.to_string();

        info!("Contract deployed at: {}", &contract_id);

        Ok(contract_id)
    }

    pub async fn get_contract_id(
        wallet_path: &str,
        contract_bin_path: &str,
    ) -> Result<(WalletUnlocked, Bech32ContractId), Box<dyn std::error::Error>> {
        let filter = match std::env::var_os("RUST_LOG") {
            Some(_) => {
                EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
            }
            None => EnvFilter::new("info"),
        };

        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .try_init();

        let mut wallet =
            WalletUnlocked::load_keystore(wallet_path, defaults::WALLET_PASSWORD, None)
                .unwrap();

        let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();

        wallet.set_provider(provider.clone());

        let _compiled = Contract::load_contract(contract_bin_path, &None).unwrap();

        let contract_id = Contract::deploy(
            contract_bin_path,
            &wallet,
            tx_params(),
            StorageConfiguration::default(),
        )
        .await
        .unwrap();

        info!("Using contract at {:?}", &contract_id);

        Ok((wallet, contract_id))
    }
}
