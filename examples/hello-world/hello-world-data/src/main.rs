use clap::Parser;
use fuel_indexer_tests::{
    defaults,
    fixtures::{get_contract_id_with_host, tx_params},
};
use fuels::prelude::SizedAsciiString;
use fuels_abigen_macro::abigen;
use rand::{seq::SliceRandom, Rng};
use std::path::{Path, PathBuf};

abigen!(
    Greet,
    "examples/hello-world/contracts/greeting/out/debug/greeting-abi.json"
);

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer test web api", about = "Test")]
pub struct Args {
    #[clap(long, help = "Test wallet filepath")]
    pub chain_config: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub contract_bin: Option<PathBuf>,
    #[clap(long, help = "Host at which to bind.")]
    pub host: Option<String>,
}

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
    let opts = Args::from_args();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;

    let names = vec![
        "Rashad", "Ava", "Noel", "James", "Ciara", "Penny", "Thompson", "Alex",
    ];

    let greetings = vec!["Hi", "Ciao", "Hola", "Buenos dias", "Bonjour", "Whatsup"];

    let chain_config = opts.chain_config.unwrap_or(
        Path::new(&manifest_dir)
            .join("..")
            .join("..")
            .join("..")
            .join("packages")
            .join("fuel-indexer-tests")
            .join("assets")
            .join("test-chain-config.json"),
    );

    let contract_bin = opts.contract_bin.unwrap_or(
        Path::new(&manifest_dir)
            .join("..")
            .join("contracts")
            .join("greeting")
            .join("out")
            .join("debug")
            .join("greeting.bin"),
    );

    let host = opts.host.unwrap_or(defaults::FUEL_NODE_ADDR.to_string());
    println!("Using Fuel node at {}", host);

    let (wallet, contract_id) = get_contract_id_with_host(
        chain_config.as_os_str().to_str().unwrap(),
        contract_bin.as_os_str().to_str().unwrap(),
        host,
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
