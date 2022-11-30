use fuel_indexer_tests::fixtures::{get_contract_id, tx_params};
use fuels::prelude::SizedAsciiString;
use fuels_abigen_macro::abigen;
use rand::{seq::SliceRandom, Rng};
use std::path::Path;

abigen!(
    Greet,
    "examples/hello-world/contracts/greeting/out/debug/greeting-abi.json"
);

static MAX_BIGINT: u64 = 0x7fffffffffffffff;
const BYTES32_LEN: usize = 0x20;

// The FuelVM only recognizes SizedAsciiStrings, but we don't always care
// about perfectly sized Strings, so we pad any String shorter than the expected
// size with whitespace. We can always trim any whitespace before saving to the
// database.
fn rightpad_whitespace(s: &str, n: usize) -> String {
    format!("{:0width$}", s, width = n)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;

    let names = vec![
        "Rashad", "Ava", "Noel", "James", "Ciara", "Penny", "Thompson", "Alex",
    ];

    let greetings = vec!["Hi", "Ciao", "Hola", "Buenos dias", "Bonjour", "Whatsup"];

    let wallet_path = Path::new(&manifest_dir)
        .join("..")
        .join("..")
        .join("..")
        .join("packages")
        .join("fuel-indexer-tests")
        .join("assets")
        .join("test-chain-config.json");

    let contract_bin_path = Path::new(&manifest_dir)
        .join("..")
        .join("contracts")
        .join("greeting")
        .join("out")
        .join("debug")
        .join("greeting.bin");

    let (wallet, contract_id) = get_contract_id(
        wallet_path.as_os_str().to_str().unwrap(),
        contract_bin_path.as_os_str().to_str().unwrap(),
    )
    .await?;

    let contract = Greet::new(contract_id, wallet.clone());

    let mut rng = rand::thread_rng();

    let id = rng.gen_range(0..MAX_BIGINT) as u64;
    let greeting = SizedAsciiString::<BYTES32_LEN>::new(rightpad_whitespace(
        greetings.choose(&mut rand::thread_rng()).unwrap(),
        BYTES32_LEN,
    ))?;
    let name = SizedAsciiString::<BYTES32_LEN>::new(rightpad_whitespace(
        names.choose(&mut rand::thread_rng()).unwrap(),
        BYTES32_LEN,
    ))?;

    let _ = contract
        .methods()
        .new_greeting(id, greeting, name)
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    Ok(())
}
