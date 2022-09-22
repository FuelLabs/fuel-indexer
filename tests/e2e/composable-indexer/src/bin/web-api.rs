use async_std::sync::{Arc, Mutex};
use axum::{extract::Extension, routing::post, Router};
use clap::Parser;
use composable_indexer::defaults;
use fuels::prelude::{Contract, Provider, TxParameters, WalletUnlocked};
use fuels_abigen_macro::abigen;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
}

abigen!(
    Message,
    "tests/e2e/composable-indexer/composable-indexer-lib/contracts/ping/out/debug/ping-abi.json"
);

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer test web api", about = "Test")]
pub struct Args {
    #[clap(long, help = "Test node host")]
    pub fuel_node_host: Option<String>,
    #[clap(long, help = "Test node port")]
    pub fuel_node_port: Option<String>,
    #[clap(long, help = "Test wallet filepath")]
    pub wallet_path: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub bin_path: Option<PathBuf>,
}

#[axum_macros::debug_handler]
async fn ping(contract: Extension<Arc<Mutex<Message>>>) -> String {
    let contract = contract.lock().await;
    let _call = contract.ping().tx_params(tx_params());
    // higher-ranked lifetime error!
    //let result = _call.call().await.unwrap();
    //let pong: Ping = result.value;
    //pong.value.to_string()
    "pong".into()
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
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    let opts = Args::from_args();

    let fuel_node_host = opts
        .fuel_node_host
        .unwrap_or_else(|| defaults::FUEL_NODE_HOST.to_string());

    let fuel_node_port = opts
        .fuel_node_port
        .unwrap_or_else(|| defaults::FUEL_NODE_PORT.to_string());

    let fuel_node_addr = format!("{}:{}", fuel_node_host, fuel_node_port);

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let provider = Provider::connect(&fuel_node_addr).await.unwrap();

    let wallet_path = opts
        .wallet_path
        .unwrap_or_else(|| Path::new(&manifest_dir).join("wallet.json"));

    info!("Wallet keystore at: {}", wallet_path.display());

    let wallet =
        WalletUnlocked::load_keystore(&wallet_path, defaults::WALLET_PASSWORD, Some(provider))?;

    info!("Connected to fuel client at {}", fuel_node_addr);

    let contract: Message =
        MessageBuilder::new(defaults::PING_CONTRACT_ID.to_string(), wallet).build();

    let bin_path = opts.bin_path.unwrap_or_else(|| {
        Path::join(
            manifest_dir,
            "composable-indexer-lib/contracts/ping/out/debug/ping.bin",
        )
    });

    let compiled =
        Contract::load_sway_contract(&bin_path.into_os_string().into_string().unwrap(), &None)
            .unwrap();
    let (id, _) = Contract::compute_contract_id_and_state_root(&compiled);
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
