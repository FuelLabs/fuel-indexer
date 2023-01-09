use clap::Parser;
use fuel_indexer_tests::{defaults, fixtures::setup_test_fuel_node};
use fuels_abigen_macro::abigen;
use std::path::{Path, PathBuf};

abigen!(
    FuelIndexer,
    "packages/fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer test web api", about = "Test")]
pub struct Args {
    #[clap(long, help = "Test wallet filepath")]
    pub wallet_path: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub contract_bin_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Args::from_args();

    let wallet_path = opts.wallet_path.unwrap_or_else(|| {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not found.");
        Path::new(&manifest_dir)
            .join("..")
            .join("..")
            .join("assets")
            .join("test-chain-config.json")
    });

    let contract_bin_path = opts.contract_bin_path.unwrap_or_else(|| {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not found.");
        Path::new(&manifest_dir)
            .join("..")
            .join("..")
            .join("contracts")
            .join("fuel-indexer-test")
            .join("out")
            .join("debug")
            .join("fuel-indexer-test.bin")
    });

    let _contract_id = setup_test_fuel_node(
        wallet_path.as_os_str().to_str().unwrap(),
        contract_bin_path.as_os_str().to_str().unwrap(),
    )
    .await?;
    std::thread::sleep(defaults::SLEEP);

    Ok(())
}
