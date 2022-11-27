use clap::Parser;
use dashboard_example::helpers::generate_multi_wallet_config;
use fuels::{
    prelude::{Config, Contract, TxParameters},
    test_helpers::launch_custom_provider_and_get_wallets,
};
use fuels_abigen_macro::abigen;
use fuels_core::parameters::StorageConfiguration;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    Dashboard,
    "examples/dashboard/contracts/out/debug/dashboard-abi.json"
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

    let wallets_config = generate_multi_wallet_config();
    let provider_config = Some(Config {
        utxo_validation: false,
        addr: "127.0.0.1:4000".parse().unwrap(),
        ..Config::local_node()
    });
    let wallets =
        launch_custom_provider_and_get_wallets(wallets_config, provider_config, None)
            .await;

    let opts = Args::from_args();
    let bin_path = opts.bin_path.unwrap_or_else(|| {
        Path::join(manifest_dir, "../../contracts/out/debug/dashboard.bin")
    });

    let bin_path_str = bin_path.as_os_str().to_str().unwrap();
    let _compiled = Contract::load_contract(bin_path_str, &None).unwrap();

    let contract_id = Contract::deploy(
        bin_path_str,
        &wallets[0],
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let contract_id = contract_id.to_string();

    info!("Contract deployed at: {}", &contract_id);

    std::thread::sleep(Duration::from_secs(60 * 60 * 10));

    Ok(())
}
