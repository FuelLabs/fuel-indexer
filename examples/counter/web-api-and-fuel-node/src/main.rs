#[macro_use]
extern crate log;

extern crate pretty_env_logger;

use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use async_mutex::Mutex;
use fuel_core::{
    chain_config::{ChainConfig, StateConfig},
    service::DbType,
};
use fuels::{
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, WalletUnlocked,
        Provider, TxParameters, DEFAULT_COIN_AMOUNT,
    },
    signers::Signer,
};
use fuels_core::parameters::StorageConfiguration;
use fuels_abigen_macro::abigen;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
}

abigen!(
    Counter,
    "examples/counter/contracts/counter/out/debug/counter-abi.json"
);

async fn get_contract_id(wallet: &WalletUnlocked) -> String {
    dotenv::dotenv().ok();
    debug!("Creating new deployment for non-existent contract");

    let _compiled =
        Contract::load_sway_contract("./../contracts/counter/out/debug/counter.bin", &None).unwrap();

    let bin_path = "./../contracts/counter/out/debug/counter.bin".to_string();
    let contract_id = Contract::deploy(&bin_path, wallet, tx_params(), StorageConfiguration::default())
        .await
        .unwrap();

    contract_id.to_string()
}

async fn setup_provider_and_wallet() -> (Provider, WalletUnlocked) {
    let mut wallet = WalletUnlocked::new_random(None);

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

    let (client, _) = setup_test_client(coins, vec![], Some(config), None).await;

    info!("Fuel client started at {:?}", client);

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    (provider, wallet)
}

#[derive(Debug, Serialize, Deserialize)]
struct InitCountResponse {
    success: bool,
    count: u64,
}

async fn initialize_count(req: HttpRequest) -> Result<HttpResponse, Error> {
    let state = match req.app_data::<web::Data<Mutex<Counter>>>() {
        Some(state) => state,
        None => {
            return Ok(HttpResponse::Ok().content_type("application/json").body(
                serde_json::to_string(&InitCountResponse {
                    success: false,
                    count: 0,
                })
                .unwrap(),
            ))
        }
    };

    let contract = state.lock().await;

    let result = contract
        .init_counter(1)
        .tx_params(tx_params())
        .call()
        .await
        .unwrap();

    debug!("Call result: {:?}", result);

    let count: u64 = result.receipts[2].val().unwrap();
    Ok(HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&InitCountResponse {
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
    let contract: Counter = CounterBuilder::new(contract_id, wallet).build();

    let state = web::Data::new(Mutex::new(contract));

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/count")
                    .app_data(web::JsonConfig::default().limit(1024))
                    .route(web::post().to(initialize_count)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
