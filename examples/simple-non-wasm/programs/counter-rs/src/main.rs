#[macro_use]
extern crate log;

extern crate pretty_env_logger;

use actix_web::{middleware, web, web::Bytes, App, Error, HttpRequest, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Mutex;
use fuels::{
    signers::Signer,
    node::{
        chain_config::{ChainConfig, StateConfig},
        service::DbType,
    },
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract,
        LocalWallet, Provider, TxParameters, DEFAULT_COIN_AMOUNT,
    },
    signers::wallet::Wallet,
};
use fuels_abigen_macro::abigen;

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price), None)
}

abigen!(Counter, "./../counter/out/debug/counter-abi.json");

async fn get_contract_id(wallet: &Wallet) -> String {
    dotenv::dotenv().ok();
    debug!("Creating new deployment for non-existent contract");
    let _compiled = Contract::load_sway_contract("./../counter/out/debug/counter.bin").unwrap();

    let bin_path = "./../counter/out/debug/counter.bin".to_string();
    let contract_id = Contract::deploy(&bin_path, wallet, tx_params())
        .await
        .unwrap();

    contract_id.to_string()
}

async fn setup_provider_and_wallet() -> (Provider, Wallet) {

    let mut wallet = LocalWallet::new_random(None);

    let number_of_coins = 11;
    let asset_id = AssetId::zeroed();
    let coins = setup_single_asset_coins(wallet.address(), asset_id, number_of_coins, DEFAULT_COIN_AMOUNT);

    let config = Config {
        chain_conf: ChainConfig {
            initial_state: Some(StateConfig {
                ..StateConfig::default()
            }),
            ..ChainConfig::local_testnet()
        },
        database_type: DbType::InMemory,
        utxo_validation: false,
        addr: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 4000),
        ..Config::local_node()
    };

    let (client, _) = setup_test_client(coins, config).await;

    info!("Fuel client started at {:?}", client);

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    (provider, wallet)
}

#[derive(Debug, Serialize, Deserialize)]
struct IncrementCountResponse {
    success: bool,
    count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct IncrementCountRequest {
    count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetCountResponse {
    success: bool,
    count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitCountResponse {
    success: bool,
    count: u64,
}

async fn initialize_count(req: HttpRequest) -> Result<HttpResponse, Error> {
    let state = req.app_data::<web::Data<Mutex<Counter>>>().unwrap();
    let contract = match state.lock() {
        Ok(c) => c,
        Err(e) => {
            error!("Could not get state: {}", e);
            return Ok(HttpResponse::Ok().content_type("application/json").body(
                serde_json::to_string(&InitCountResponse {
                    success: false,
                    count: 1,
                })
                .unwrap(),
            ));
        }
    };

    let result = contract
        .init_counter(1)
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    debug!("{:?}", result);

    // let count: u64 = result.receipts[2].val().unwrap();
    let count = 1;
    Ok(HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&InitCountResponse {
            success: true,
            count,
        })
        .unwrap(),
    ))
}

async fn increment_count(req: HttpRequest, body: Bytes) -> Result<HttpResponse, Error> {
    let json_body: IncrementCountRequest = serde_json::from_slice(&body).unwrap();
    let state = req.app_data::<web::Data<Mutex<Counter>>>().unwrap();

    let contract = match state.lock() {
        Ok(c) => c,
        Err(e) => {
            error!("Could not get state: {}", e);
            return Ok(HttpResponse::Ok().content_type("application/json").body(
                serde_json::to_string(&IncrementCountResponse {
                    success: false,
                    count: 0u64,
                })
                .unwrap(),
            ));
        }
    };

    let result = contract
        .increment_counter(json_body.count)
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    debug!("{:?}", result);
    let count: u64 = result.receipts[1].val().unwrap();

    Ok(HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&IncrementCountResponse {
            success: true,
            count,
        })
        .unwrap(),
    ))
}

async fn get_count(req: HttpRequest) -> Result<HttpResponse, Error> {
    let state = req.app_data::<web::Data<Mutex<Counter>>>().unwrap();
    let contract = state.lock().unwrap();

    let result = contract
        .get_count()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    debug!("{:?}", result);
    let count: u64 = result.receipts[1].val().unwrap();

    Ok(HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&GetCountResponse {
            success: true,
            count,
        })
        .unwrap(),
    ))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let (_provider, wallet) = setup_provider_and_wallet().await;
    let contract_id: String = get_contract_id(&wallet).await;
    info!("Using contract at {}", contract_id);
    let contract: Counter = Counter::new(contract_id, wallet);

    let state = web::Data::new(Mutex::new(contract));

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/count")
                    .app_data(web::JsonConfig::default().limit(1024))
                    .route(web::put().to(increment_count))
                    .route(web::post().to(initialize_count))
                    .route(web::get().to(get_count)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};

    #[actix_web::test]
    async fn test_increment_count() {
        let app = test::init_service(
            App::new().service(web::resource("/counter").route(web::post().to(increment_count))),
        )
        .await;

        let req = test::TestRequest::post().uri("/counter").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body_bytes, r##"{"success":true}"##);
    }

    #[actix_web::test]
    async fn test_get_count() {
        let app = test::init_service(
            App::new().service(web::resource("/count").route(web::get().to(get_count))),
        )
        .await;

        let req = test::TestRequest::get().uri("/count").to_request();
        let resp = app.call(req).await.unwrap();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body_bytes, r##"{"success":true,"count":1}"##);
    }
}

