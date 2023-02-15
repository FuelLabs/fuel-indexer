use clap::Parser;
use fuel_indexer_tests::fixtures::setup_test_fuel_node;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "hello-world-node",
    about = "Test program used to start up a test Fuel node for the hello-world example."
)]
pub struct Args {
    #[clap(long, help = "Test wallet filepath")]
    pub chain_config: Option<PathBuf>,
    #[clap(long, help = "Host at which to bind.")]
    pub host: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Args::from_args();

    let chain_config = opts.chain_config.unwrap_or_else(|| {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packages")
            .join("fuel-indexer-tests")
            .join("assets")
            .join("test-chain-config.json")
    });

    let host = if opts.host.is_some() {
        opts.host
    } else {
        Some("127.0.0.1".to_string())
    };

    println!("Spinning up test Fuel node; node will automatically exit in ten minutes.");
    let server_handle = tokio::spawn(setup_test_fuel_node(chain_config, None, host));
    std::thread::sleep(std::time::Duration::from_secs(600));

    server_handle.abort();

    Ok(())
}
