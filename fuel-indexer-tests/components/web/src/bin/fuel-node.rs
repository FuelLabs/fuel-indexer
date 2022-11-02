use clap::Parser;
use fuel_indexer_tests::{defaults, fixtures::tx_params};
use fuels::{
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, Provider,
        WalletUnlocked, DEFAULT_COIN_AMOUNT,
    },
    signers::Signer,
    core::parameters::StorageConfiguration,
};
use fuels_abigen_macro::abigen;
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    FuelIndexer,
    "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer test web api", about = "Test")]
pub struct Args {
    #[clap(long, help = "Test wallet filepath")]
    pub wallet_path: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub bin_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        let p = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        p.display().to_string()
    });

    let manifest_dir = Path::new(&manifest_dir);

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

    let opts = Args::from_args();
    let wallet_path = opts
        .wallet_path
        .unwrap_or_else(|| Path::new(&manifest_dir).join("wallet.json"));

    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

    let mut wallet =
        WalletUnlocked::load_keystore(&wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    info!(
        "Wallet({}) keystore at: {}",
        wallet.address(),
        wallet_path.display()
    );

    let bin_path = opts.bin_path.unwrap_or_else(|| {
        Path::join(
            manifest_dir,
            "../../contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin",
        )
    });

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

    let (client, _) = setup_test_client(coins, vec![], Some(config), None).await;

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
