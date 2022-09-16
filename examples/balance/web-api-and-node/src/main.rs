#[macro_use]
extern crate log;

extern crate pretty_env_logger;

use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use async_mutex::Mutex;
use fuels::{
    node::{
        chain_config::{ChainConfig, StateConfig},
        service::DbType,
    },
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, LocalWallet,
        Provider, TxParameters, DEFAULT_COIN_AMOUNT,
    },
    signers::wallet::Wallet,
    signers::Signer,
};
use fuels_abigen_macro::abigen;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price), None)
}

abigen!(
    Balance,
    "examples/balance/contracts/balance/out/debug/balance-abi.json"
);

async fn get_contract_id(wallet: &Wallet) -> String {
    dotenv::dotenv().ok();
    debug!("Creating new deployment for non-existent contract");

    let _compiled =
        Contract::load_sway_contract("./../contracts/balance/out/debug/balance.bin").unwrap();

    let bin_path = "./../contracts/balance/out/debug/balance.bin".to_string();
    let contract_id = Contract::deploy(&bin_path, wallet, tx_params())
        .await
        .unwrap();

    contract_id.to_string()
}

async fn setup_provider_and_wallet() -> (Provider, Wallet) {
    let mut wallet = LocalWallet::new_random(None);

    let number_of_coins = 11;
    let asset_id = AssetId::zeroed();
    let coins = setup_single_asset_coins(
        wallet.address(),
        asset_id,
        number_of_coins,
        DEFAULT_COIN_AMOUNT,
    );

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
struct InitBalanceResponse {
    success: bool,
    balance: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BalanceResponse {
    success: bool,
    balance: u64,
}

async fn initialize_balance(req: HttpRequest) -> Result<HttpResponse, Error> {
    let state = match req.app_data::<web::Data<Mutex<Balance>>>() {
        Some(state) => state,
        None => {
            return Ok(HttpResponse::Ok().content_type("application/json").body(
                serde_json::to_string(&InitBalanceResponse {
                    success: false,
                    balance: 0,
                })
                .unwrap(),
            ))
        }
    };

    let contract = state.lock().await;

    let result = contract
        .init_balance()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    info!("call result: {:?}", result.receipts);

    let balance: u64 = result.receipts[1].data().unwrap()[15].into();
    Ok(HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&InitBalanceResponse {
            success: true,
            balance,
        })
        .unwrap(),
    ))
}

async fn get_balance(req: HttpRequest) -> Result<HttpResponse, Error> {
    let state = match req.app_data::<web::Data<Mutex<Balance>>>() {
        Some(state) => state,
        None => {
            return Ok(HttpResponse::Ok().content_type("application/json").body(
                serde_json::to_string(&BalanceResponse {
                    success: false,
                    balance: 0,
                })
                .unwrap(),
            ))
        }
    };

    let contract = state.lock().await;
    let result = contract
        .get_balance()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    info!("call result: {:?}", result.receipts);

    let balance: u64 = result.receipts[1].data().unwrap()[15].into();
    Ok(HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&BalanceResponse {
            success: true,
            balance,
        })
        .unwrap(),
    ))
}

async fn liquidate(req: HttpRequest) -> Result<HttpResponse, Error> {
    let state = match req.app_data::<web::Data<Mutex<Balance>>>() {
        Some(state) => state,
        None => {
            return Ok(HttpResponse::Ok().content_type("application/json").body(
                serde_json::to_string(&BalanceResponse {
                    success: false,
                    balance: 0,
                })
                .unwrap(),
            ))
        }
    };

    let contract = state.lock().await;
    let result = contract
        .liquidate_balance()
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    info!("call result: {:?}", result.receipts);

    let balance: u64 = result.receipts[1].data().unwrap()[15].into();
    Ok(HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&BalanceResponse {
            success: true,
            balance,
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
    let contract: Balance = Balance::new(contract_id, wallet);

    let state = web::Data::new(Mutex::new(contract));

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/init_balance")
                    .app_data(web::JsonConfig::default().limit(1024))
                    .route(web::post().to(initialize_balance)),
            )
            .service(
                web::resource("/balance")
                    .app_data(web::JsonConfig::default().limit(1024))
                    .route(web::post().to(get_balance)),
            )
            .service(
                web::resource("/liquidate_balance")
                    .app_data(web::JsonConfig::default().limit(1024))
                    .route(web::post().to(liquidate)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
