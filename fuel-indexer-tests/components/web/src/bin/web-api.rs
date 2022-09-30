use async_std::sync::{Arc, Mutex};
use axum::{extract::Extension, routing::post, Router};
use clap::Parser;
use fuel_indexer_lib::utils::derive_socket_addr;
use fuel_indexer_tests::{defaults, utils::tx_params};
use fuels::prelude::{Contract, Provider, WalletUnlocked};
use fuels_abigen_macro::abigen;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    FuelIndexer,
    "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer test web api", about = "Test")]
pub struct Args {
    #[clap(long, default_value = "0.0.0.0", help = "Test node host")]
    pub fuel_node_host: String,
    #[clap(long, default_value = "4000", help = "Test node port")]
    pub fuel_node_port: String,
    #[clap(long, help = "Test wallet filepath")]
    pub wallet_path: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub bin_path: Option<PathBuf>,
}

async fn ping(Extension(contract): Extension<Arc<Mutex<FuelIndexer>>>) -> String {
    let contract = contract.lock().await;
    let result = contract.ping().tx_params(tx_params()).call().await.unwrap();
    let ping: Ping = result.value;
    ping.value.to_string()
}

async fn pong(Extension(contract): Extension<Arc<Mutex<FuelIndexer>>>) -> String {
    let contract = contract.lock().await;
    let result = contract.pong().tx_params(tx_params()).call().await.unwrap();
    let pong: Pong = result.value;
    pong.value.to_string()
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

    let opts = Args::from_args();

    let fuel_node_addr =
        derive_socket_addr(&opts.fuel_node_host, &opts.fuel_node_port).unwrap();

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let provider = Provider::connect(fuel_node_addr.to_string()).await.unwrap();

    let wallet_path = opts
        .wallet_path
        .unwrap_or_else(|| Path::new(&manifest_dir).join("wallet.json"));

    info!("Wallet keystore at: {}", wallet_path.display());

    let wallet = WalletUnlocked::load_keystore(
        &wallet_path,
        defaults::WALLET_PASSWORD,
        Some(provider),
    )?;

    info!("Connected to fuel client at {}", fuel_node_addr);

    let contract: FuelIndexer =
        FuelIndexerBuilder::new(defaults::PING_CONTRACT_ID.to_string(), wallet).build();

    let bin_path = opts.bin_path.unwrap_or_else(|| {
        Path::join(
            manifest_dir,
            "../../contracts/fuel-indexer-test/out/debug/fuel-indexer.bin",
        )
    });

    let compiled = Contract::load_sway_contract(
        &bin_path.into_os_string().into_string().unwrap(),
        &None,
    )
    .unwrap();
    let (id, _) = Contract::compute_contract_id_and_state_root(&compiled);
    info!("Using contract at {}", id);

    let state = Arc::new(Mutex::new(contract));

    info!("Starting server at {}", defaults::WEB_API_ADDR);

    let app = Router::new()
        .route("/ping", post(ping))
        .layer(Extension(state.clone()))
        .route("/pong", post(pong))
        .layer(Extension(state.clone()));

    let addr: SocketAddr = defaults::WEB_API_ADDR.parse().unwrap();

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Service failed to start");

    Ok(())
}
