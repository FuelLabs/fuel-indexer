use clap::Parser;
use fuel_indexer_tests::{defaults, fixtures::setup_test_fuel_node};
use fuels_macros::abigen;
use std::path::{Path, PathBuf};

abigen!(Contract(
    name = "FuelIndexerTest",
    abi = "packages/fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
));

#[derive(Debug, Parser, Clone)]
#[clap(name = "fuel-node", about = "An ephemeral Fuel node used for testing.")]
pub struct Args {
    #[clap(long, help = "Test wallet filepath")]
    pub chain_config: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub contract_bin: Option<PathBuf>,
    #[clap(long, help = "Host at which to bind.")]
    pub host: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Args::from_args();

    let chain_config = opts.chain_config.unwrap_or_else(|| {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not found.");
        Path::new(&manifest_dir)
            .join("..")
            .join("..")
            .join("assets")
            .join("test-chain-config.json")
    });

    let contract_bin = opts.contract_bin.unwrap_or_else(|| {
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

    let host = opts
        .host
        .unwrap_or_else(|| defaults::FUEL_NODE_ADDR.to_string());

    let _contract_id = setup_test_fuel_node(chain_config, Some(contract_bin), Some(host))
        .await
        .unwrap();
    std::thread::sleep(defaults::SLEEP);

    Ok(())
}
