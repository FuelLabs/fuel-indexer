use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Arc;
use clap::Parser;
use fuel_indexer_tests::{defaults, fixtures::tx_params};
use fuels::{
    prelude::{Contract, Provider, WalletUnlocked},
    signers::Signer,
};
use fuels_abigen_macro::abigen;
use fuels_core::parameters::StorageConfiguration;
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    FuelIndexerTest,
    "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer test web api", about = "Test")]
pub struct Args {
    #[clap(long, default_value = defaults::FUEL_NODE_HOST, help = "Test node host")]
    pub fuel_node_host: String,
    #[clap(long, default_value = defaults::FUEL_NODE_PORT, help = "Test node port")]
    pub fuel_node_port: String,
    #[clap(long, help = "Test wallet filepath")]
    pub wallet_path: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub bin_path: Option<PathBuf>,
}

async fn fuel_indexer_test_blocks(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .trigger_ping()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_ping(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .trigger_ping()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

// FIXME: This errors and reverts -- how to do a simple transfer without reversion?
async fn fuel_indexer_test_transfer(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .trigger_transfer()
        .tx_params(tx_params())
        .call()
        .await;

    HttpResponse::Ok()
}

async fn fuel_indexer_test_log(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .trigger_log()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_logdata(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .trigger_logdata()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_scriptresult(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .trigger_scriptresult()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

pub struct AppState {
    pub contract: FuelIndexerTest,
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

    let opts = Args::from_args();
    let wallet_path = opts
        .wallet_path
        .unwrap_or_else(|| Path::new(&manifest_dir).join("wallet.json"));

    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

    let mut wallet =
        WalletUnlocked::load_keystore(&wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();

    wallet.set_provider(provider.clone());

    info!(
        "Wallet({}) keystore at: {}",
        wallet.address(),
        wallet_path.display()
    );

    let bin_path = opts.bin_path.unwrap_or_else(|| {
        Path::join(
            manifest_dir,
            "../../contracts/fuel-indexer-test/out/debug/fuel-indexer-test.bin",
        )
    });

    let bin_path_str = bin_path.as_os_str().to_str().unwrap();
    let _compiled = Contract::load_sway_contract(bin_path_str, &None).unwrap();

    let contract_id = Contract::deploy(
        bin_path_str,
        &wallet,
        tx_params(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let contract_id = contract_id.to_string();

    info!("Using contract at {}", contract_id);

    let contract =
        FuelIndexerTestBuilder::new(contract_id.to_string(), wallet.clone()).build();

    info!("Starting server at {}", defaults::WEB_API_ADDR);

    let state = web::Data::new(Arc::new(AppState { contract }));

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/block", web::post().to(fuel_indexer_test_blocks))
            .route("/ping", web::post().to(fuel_indexer_test_ping))
            .route("/transfer", web::post().to(fuel_indexer_test_transfer))
            .route("/log", web::post().to(fuel_indexer_test_log))
            .route("/logdata", web::post().to(fuel_indexer_test_logdata))
            .route("/scriptresult", web::post().to(fuel_indexer_test_scriptresult))
    })
    .bind(defaults::WEB_API_ADDR)
    .unwrap()
    .run()
    .await;

    Ok(())
}
