use fuel_indexer_tests::{defaults, fixtures::tx_params, WORKSPACE_ROOT};
use fuels::{
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, Provider,
        WalletUnlocked, DEFAULT_COIN_AMOUNT,
    },
    signers::Signer,
};
use fuels_abigen_macro::abigen;
use fuels_core::parameters::StorageConfiguration;
use std::path::Path;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    FuelIndexer,
    "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let wallet_path = Path::new(WORKSPACE_ROOT).join("assets").join("wallet.json");

    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    info!(
        "Wallet({}) keystore at: {}",
        wallet.address(),
        wallet_path.display()
    );

    let bin_path = Path::new(WORKSPACE_ROOT)
        .join("contracts")
        .join("fuel-indexer-test")
        .join("out")
        .join("debug")
        .join("fuel-indexer-test.bin");

    let bin_path_str = bin_path.as_os_str().to_str().unwrap();
    let _compiled = Contract::load_contract(bin_path_str, &None).unwrap();

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

    let (client, _) = setup_test_client(coins, vec![], Some(config), None, None).await;

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    let contract_id = Contract::deploy(
        bin_path_str,
        &wallet,
        tx_params(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let contract_id = contract_id.to_string();

    info!("Contract deployed at: {}", &contract_id);

    std::thread::sleep(defaults::SLEEP);

    Ok(())
}
