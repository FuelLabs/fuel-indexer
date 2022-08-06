use composable_indexer_test::{defaults, tx_params};
use fuels::{
    node::{
        chain_config::{ChainConfig, StateConfig},
        service::DbType,
    },
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, LocalWallet,
        Provider, DEFAULT_COIN_AMOUNT,
    },
    signers::Signer,
};
use fuels_abigen_macro::abigen;
use std::path::Path;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    Message,
    "tests/e2e/composable-indexer-test/indexer/contracts/ping/out/debug/ping-abi.json"
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Manifest dir unknown");

    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let wallet_path = Path::new(&manifest_dir).join("wallet.json");

    info!("Wallet keystore at: {}", wallet_path.display());

    let mut wallet = LocalWallet::load_keystore(&wallet_path, defaults::WALLET_PASSWORD, None)?;

    info!("Using wallet address at: {}", wallet.address());

    let number_of_coins = 11;
    let asset_id = AssetId::zeroed();
    let coins = setup_single_asset_coins(
        wallet.address(),
        asset_id,
        number_of_coins,
        DEFAULT_COIN_AMOUNT,
    );

    let config = Config {
        chain_conf: ChainConfig {
            initial_state: Some(StateConfig {
                ..StateConfig::default()
            }),
            ..ChainConfig::local_testnet()
        },
        database_type: DbType::InMemory,
        utxo_validation: false,
        addr: defaults::FUEL_NODE_ADDR.parse().unwrap(),
        ..Config::local_node()
    };

    let (client, _) = setup_test_client(coins, config).await;

    info!("Fuel client started at {:?}", client);

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    let bin_path = "indexer/contracts/ping/out/debug/ping.bin".to_string();

    let contract_id = Contract::deploy(&bin_path, &wallet, tx_params())
        .await
        .unwrap();

    let contract_id = contract_id.to_string();

    info!("Contract deployed at: {}", &contract_id);

    std::thread::sleep(defaults::SLEEP);

    Ok(())
}
