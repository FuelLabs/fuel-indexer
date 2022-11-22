use fuel_indexer_tests::{defaults, fixtures::tx_params};
use fuels::{
    core::types::SizedAsciiString,
    prelude::{Contract, Provider, WalletUnlocked},
};
use fuels_abigen_macro::abigen;
use fuels_core::parameters::StorageConfiguration;
use rand::Rng;
use std::path::Path;

abigen!(
    Greet,
    "examples/hello-world/contracts/greeting/out/debug/greeting-abi.json"
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find project root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let project_root = std::fs::canonicalize(
        &Path::new(&manifest_dir).join("..").join("..").join(".."),
    )?;

    // Load wallet and set provider
    let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();

    let wallet_path = Path::new(&project_root)
        .join("fuel-indexer-tests")
        .join("assets")
        .join("test-chain-config.json");

    let mut wallet =
        WalletUnlocked::load_keystore(&wallet_path, defaults::WALLET_PASSWORD, None)
            .unwrap();

    wallet.set_provider(provider.clone());

    // Load compiled contract and deploy
    let bin_path = Path::new(&project_root)
        .join("examples")
        .join("hello-world")
        .join("contracts")
        .join("greeting")
        .join("out")
        .join("debug")
        .join("greeting.bin");

    let _ = Contract::load_contract(bin_path.to_str().unwrap(), &None).unwrap();

    let contract_id = Contract::deploy(
        bin_path.to_str().unwrap(),
        &wallet,
        tx_params(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    println!("\nContract deployed at: '{}'\n", contract_id);

    let contract = Greet::new(contract_id, wallet.clone());

    // Call the contract
    let id = rand::thread_rng().gen_range(0..100000) as u64;
    let name = SizedAsciiString::new("Fuel".to_string())?;
    let greeting = SizedAsciiString::new("Benvenudo".to_string())?;

    contract
        .methods()
        .new_greeting(id, greeting, name)
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    Ok(())
}
