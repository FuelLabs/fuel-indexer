use async_std::sync::{Arc, Mutex};
use axum::{extract::Extension, routing::post, Router};
use composable_indexer_test::defaults;
use composable_indexer_test::defaults::{FUEL_NODE_ADDR, PING_CONTRACT_ID};
use fuels::prelude::{Contract, LocalWallet, Provider, TxParameters};
use fuels_abigen_macro::abigen;
use std::{net::SocketAddr, path::Path};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price), None)
}

abigen!(
    Message,
    "tests/e2e/composable-indexer-test/indexer/contracts/ping/out/debug/ping-abi.json"
);

#[axum_macros::debug_handler]
async fn ping(Extension(contract): Extension<Arc<Mutex<Message>>>) -> String {
    let contract = contract.lock().await;
    let result = contract.ping().tx_params(tx_params()).call().await.unwrap();
    let pong: Ping = result.value;
    pong.value.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Manifest dir unknown");

    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let provider = Provider::connect(defaults::FUEL_NODE_ADDR.parse().unwrap())
        .await
        .unwrap();

    let wallet_path = Path::new(&manifest_dir).join("wallet.json");

    info!("Wallet keystore at: {}", wallet_path.display());

    let wallet =
        LocalWallet::load_keystore(&wallet_path, defaults::WALLET_PASSWORD, Some(provider))?;

    info!("Connected to fuel client at {}", FUEL_NODE_ADDR);

    let contract = Message::new(PING_CONTRACT_ID.to_string(), wallet);
    let compiled =
        Contract::load_sway_contract("indexer/contracts/ping/out/debug/ping.bin").unwrap();
    let id = Contract::compute_contract_id(&compiled).to_string();
    info!("Using contract at {}", id);

    let state = Arc::new(Mutex::new(contract));

    info!("Starting server at {}", defaults::WEB_API_ADDR);

    let app = Router::new()
        .route("/ping", post(ping))
        .layer(Extension(state.clone()));

    let addr: SocketAddr = defaults::WEB_API_ADDR.parse().unwrap();

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Service failed to start");

    Ok(())
}
