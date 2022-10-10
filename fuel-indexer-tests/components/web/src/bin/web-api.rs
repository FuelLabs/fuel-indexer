use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Arc;
use clap::Parser;
use fuel_indexer_tests::{defaults, fixtures::tx_params};
use fuels::{
    prelude::{Contract, Provider, StorageConfiguration, WalletUnlocked},
    signers::Signer,
};
use fuels_abigen_macro::abigen;
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

async fn fuel_indexer_test_blocks(
    contract: web::Data<Arc<FuelIndexerTest>>,
) -> impl Responder {
    let _ = contract
        .methods()
        .trigger_ping()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_ping(
    contract: web::Data<Arc<FuelIndexerTest>>,
) -> impl Responder {
    let _ = contract
        .methods()
        .trigger_ping()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_transfer(
    contract: web::Data<Arc<FuelIndexerTest>>,
) -> impl Responder {
    let _ = contract
        .methods()
        .trigger_transfer()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_log(
    contract: web::Data<Arc<FuelIndexerTest>>,
) -> impl Responder {
    let _ = contract
        .methods()
        .trigger_log()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_logdata(
    contract: web::Data<Arc<FuelIndexerTest>>,
) -> impl Responder {
    let _ = contract
        .methods()
        .trigger_logdata()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
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
        WalletUnlocked::load_keystore(wallet_path_str, defaults::WALLET_PASSWORD, None)
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
    let _compiled = Contract::load_contract(bin_path_str, &None).unwrap();

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

    let contract = FuelIndexerTest::new(contract_id.to_string(), wallet);
    let contract = web::Data::new(Arc::new(contract));

    info!("Starting server at {}", defaults::WEB_API_ADDR);

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(contract.clone())
            .route("/block", web::post().to(fuel_indexer_test_blocks))
            .route("/ping", web::post().to(fuel_indexer_test_ping))
            .route("/transfer", web::post().to(fuel_indexer_test_transfer))
            .route("/log", web::post().to(fuel_indexer_test_log))
            .route("/logdata", web::post().to(fuel_indexer_test_logdata))
    })
    .bind(defaults::WEB_API_ADDR)
    .unwrap()
    .run()
    .await;

    Ok(())
}
