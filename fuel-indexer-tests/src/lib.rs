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
    pub const SLEEP: Duration = Duration::from_secs(60 * 60 * 10);
    pub const WALLET_PASSWORD: &str = "password";
}

pub mod fixtures {
    use super::defaults;
    use fuel_indexer::{
        config::{DatabaseConfig, FuelNodeConfig, GraphQLConfig, IndexerConfig},
        IndexerService, Manifest,
    };
    use fuel_indexer_database::IndexerConnectionPool;
    use fuels::{
        prelude::{
            setup_single_asset_coins, setup_test_client, AssetId, Config, Contract,
            Provider, TxParameters, WalletUnlocked, DEFAULT_COIN_AMOUNT,
        },
        signers::Signer,
    };
    use fuels_abigen_macro::abigen;
    use fuels_core::parameters::StorageConfiguration;
    use sqlx::{pool::Pool, Postgres};
    use std::path::Path;
    use tracing_subscriber::filter::EnvFilter;

    pub fn tx_params() -> TxParameters {
        let gas_price = 0;
        let gas_limit = 1_000_000;
        let byte_price = 0;
        TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
    }

    // TODO: We have this ABI JSON temporarily defined in multiple places due to (what is
    // currently believed to be) somewhat inconsistent behavior with regard to path resolution
    // in the abigen! macro. A formal Github issue is pending. The real/original ABI JSON file
    // lives under contracts/fuel-indexer-test. The others are temporary copies.
    abigen!(FuelIndexerTest, "fuel-indexer-test-abi.json");

    const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

    pub async fn postgres_connection(db_url: &str) -> Pool<Postgres> {
        match IndexerConnectionPool::connect(db_url).await.unwrap() {
            IndexerConnectionPool::Postgres(p) => p,
            _ => panic!("Should be postgres!"),
        }
    }

    pub async fn setup_indexer_service() {
        let config = IndexerConfig {
            fuel_node: FuelNodeConfig::from(
                defaults::FUEL_NODE_ADDR
                    .parse::<std::net::SocketAddr>()
                    .unwrap(),
            ),
            database: DatabaseConfig::Postgres {
                user: "postgres".into(),
                password: "my-secret".into(),
                host: "127.0.0.1".into(),
                port: "5432".into(),
                database: "postgres".to_string(),
            },
            graphql_api: GraphQLConfig::default(),
        };

        let mut indexer_service = IndexerService::new(config).await.unwrap();

        let manifest_path =
            Path::new(WORKSPACE_DIR).join("assets/fuel_indexer_test_unit.yaml");
        let manifest: Manifest = Manifest::from_file(&manifest_path).unwrap();

        indexer_service
            .register_indices(Some(manifest), true)
            .await
            .expect("Failed to initialize indexer");

        indexer_service.run().await;
    }

    pub async fn setup_test_client_and_wallet() -> WalletUnlocked {
        let filter = match std::env::var_os("RUST_LOG") {
            Some(_) => {
                EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
            }
            None => EnvFilter::new("info"),
        };

        tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .init();

        let workspace_dir = Path::new(WORKSPACE_DIR);

        let wallet_path = workspace_dir.join("assets/wallet.json");
        let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

        let mut wallet = WalletUnlocked::load_keystore(
            &wallet_path_str,
            defaults::WALLET_PASSWORD,
            None,
        )
        .unwrap();

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

        wallet.set_provider(provider);

        wallet
    }

    pub async fn setup_contract(wallet: WalletUnlocked) -> FuelIndexerTest {
        let workspace_dir = Path::new(WORKSPACE_DIR);

        let bin_path = workspace_dir
            .join("contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin");
        let bin_path_str = bin_path.as_os_str().to_str().unwrap();

        let contract_id = Contract::deploy(
            bin_path_str,
            &wallet,
            tx_params(),
            StorageConfiguration::default(),
        )
        .await
        .unwrap();

        FuelIndexerTestBuilder::new(contract_id.to_string(), wallet).build()
    }
}
