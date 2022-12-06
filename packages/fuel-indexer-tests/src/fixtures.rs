use crate::{defaults, WORKSPACE_ROOT};
use fuel_indexer::IndexerService;
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::config::{
    DatabaseConfig, FuelNodeConfig, GraphQLConfig, IndexerConfig,
};
use fuels::{
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Bech32ContractId, Config,
        Contract, Provider, TxParameters, WalletUnlocked, DEFAULT_COIN_AMOUNT,
    },
    signers::Signer,
};
use fuels_core::parameters::StorageConfiguration;
use sqlx::{pool::PoolConnection, Postgres, Sqlite};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

pub async fn postgres_connection() -> PoolConnection<Postgres> {
    let config = DatabaseConfig::Postgres {
        user: "postgres".into(),
        password: "my-secret".into(),
        host: "127.0.0.1".into(),
        port: "5432".into(),
        database: "postgres".to_string(),
    };
    match IndexerConnectionPool::connect(&config.to_string())
        .await
        .unwrap()
    {
        IndexerConnectionPool::Postgres(p) => p.acquire().await.unwrap(),
        _ => panic!("Expected Postgres connection."),
    }
}

pub fn test_sqlite_db_path() -> String {
    format!("sqlite://{}/test.db", WORKSPACE_ROOT)
}

pub async fn sqlite_connection() -> PoolConnection<Sqlite> {
    let db_url = test_sqlite_db_path();
    match IndexerConnectionPool::connect(&db_url).await.unwrap() {
        IndexerConnectionPool::Sqlite(p) => p.acquire().await.unwrap(),
        _ => panic!("Expected Sqlite connection."),
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

    let (client, _) = setup_test_client(coins, vec![], Some(config), None, None).await;

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

pub async fn indexer_service() -> IndexerService {
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
        metrics: false,
    };

    let pool = IndexerConnectionPool::connect(&config.database.to_string());

    IndexerService::new(config, pool, None).await.unwrap()
}
