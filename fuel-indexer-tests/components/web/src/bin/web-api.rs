use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Arc;
use fuel_indexer_tests::{
    defaults,
    fixtures::{get_contract_id, tx_params},
};
use fuels::prelude::CallParameters;
use fuels_abigen_macro::abigen;
use std::path::Path;
use tracing::info;

abigen!(
    FuelIndexerTest,
    "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

async fn fuel_indexer_test_blocks(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .methods()
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
        .methods()
        .trigger_ping()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_transfer(state: web::Data<Arc<AppState>>) -> impl Responder {
    let call_params = CallParameters::new(Some(1_000_000), None, None);

    let _ = state
        .contract
        .methods()
        .trigger_transfer()
        .tx_params(tx_params())
        .call_params(call_params)
        .call()
        .await
        .unwrap();

    HttpResponse::Ok()
}

async fn fuel_indexer_test_log(state: web::Data<Arc<AppState>>) -> impl Responder {
    let _ = state
        .contract
        .methods()
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
        .methods()
        .trigger_logdata()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_scriptresult(
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    let _ = state
        .contract
        .methods()
        .trigger_scriptresult()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();
    HttpResponse::Ok()
}

async fn fuel_indexer_test_transferout(
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    let call_params = CallParameters::new(Some(1_000_000), None, None);

    let _ = state
        .contract
        .methods()
        .trigger_transferout()
        .append_variable_outputs(1)
        .tx_params(tx_params())
        .call_params(call_params)
        .call()
        .await;

    HttpResponse::Ok()
}

async fn fuel_indexer_test_messageout(state: web::Data<Arc<AppState>>) -> impl Responder {
    let call_params = CallParameters::new(Some(1_000_000), None, None);

    let _ = state
        .contract
        .methods()
        .trigger_messageout()
        .append_message_outputs(1)
        .tx_params(tx_params())
        .call_params(call_params)
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
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;

    let wallet_path = Path::new(&manifest_dir)
        .join("..")
        .join("..")
        .join("assets")
        .join("test-chain-config.json");

    let contract_bin_path = Path::new(&manifest_dir)
        .join("..")
        .join("..")
        .join("contracts")
        .join("fuel-indexer-test")
        .join("out")
        .join("debug")
        .join("fuel-indexer-test.bin");

    let (wallet, contract_id) = get_contract_id(
        wallet_path.as_os_str().to_str().unwrap(),
        contract_bin_path.as_os_str().to_str().unwrap(),
    )
    .await?;

    let contract = FuelIndexerTest::new(contract_id, wallet.clone());

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
            .route(
                "/scriptresult",
                web::post().to(fuel_indexer_test_scriptresult),
            )
            .route(
                "/transferout",
                web::post().to(fuel_indexer_test_transferout),
            )
            .route("/messageout", web::post().to(fuel_indexer_test_messageout))
    })
    .bind(defaults::WEB_API_ADDR)
    .unwrap()
    .run()
    .await;

    Ok(())
}
