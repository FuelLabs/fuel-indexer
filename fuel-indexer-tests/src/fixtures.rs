use crate::{defaults, WORKSPACE_ROOT};
use fuel_indexer::IndexerService;
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::{
    config::{DatabaseConfig, FuelNodeConfig, GraphQLConfig, IndexerConfig},
    manifest::Manifest,
};
use fuels::{
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, Provider,
        TxParameters, WalletUnlocked, DEFAULT_COIN_AMOUNT,
    },
    signers::Signer,
};
use fuels_abigen_macro::abigen;
use fuels_core::parameters::StorageConfiguration;
use sqlx::{pool::PoolConnection, Postgres, Sqlite};
use std::path::{Path, PathBuf};
use tracing::info;

abigen!(
    FuelIndexerTest,
    "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

pub async fn postgres_connection() -> PoolConnection<Postgres> {
    let db_url = DatabaseConfig::default();
    match IndexerConnectionPool::connect(&db_url.to_string())
        .await
        .unwrap()
    {
        IndexerConnectionPool::Postgres(p) => p.acquire().await.unwrap(),
        _ => panic!("Expected Postgres connection."),
    }
}

pub fn test_sqlite_db_path() -> String {
    format!("sqlite://{}/test.db", WORKSPACE_ROOT)
}

pub async fn sqlite_connection_pool() -> PoolConnection<Sqlite> {
    let db_url = DatabaseConfig::Sqlite {
        path: PathBuf::from(test_sqlite_db_path()),
    };
    match IndexerConnectionPool::connect(&db_url.to_string())
        .await
        .unwrap()
    {
        IndexerConnectionPool::Sqlite(p) => p.acquire().await.unwrap(),
        _ => panic!("Expected Sqlite connection."),
    }
}

pub fn http_client() -> reqwest::Client {
    reqwest::Client::new()
}

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
}

pub async fn connect_to_deployed_contract(
) -> Result<FuelIndexerTest, Box<dyn std::error::Error>> {
    let wallet_path = Path::new(WORKSPACE_ROOT).join("assets").join("wallet.json");
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

    let bin_path = Path::new(WORKSPACE_ROOT)
        .join("contracts")
        .join("fuel-indexer-test")
        .join("out")
        .join("debug")
        .join("fuel-indexer-test.bin");

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

    info!("Using contract at {}", contract_id.to_string());

    let contract = FuelIndexerTest::new(contract_id, wallet.clone());

    info!("Starting server at {}", defaults::WEB_API_ADDR);

    Ok(contract)
}

pub async fn setup_test_client_and_deploy_contract(
) -> Result<(), Box<dyn std::error::Error>> {
    let wallet_path = Path::new(WORKSPACE_ROOT).join("assets").join("wallet.json");

    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    info!(
        "Wallet({}) keystore at: {}",
        wallet.address(),
        wallet_path.display()
    );

    let bin_path = Path::new(WORKSPACE_ROOT)
        .join("contracts")
        .join("fuel-indexer-test")
        .join("out")
        .join("debug")
        .join("fuel-indexer-test.bin");

    let bin_path_str = bin_path.as_os_str().to_str().unwrap();
    let _compiled = Contract::load_contract(bin_path_str, &None).unwrap();

    let number_of_coins = 11;
    let asset_id = AssetId::zeroed();
    let coins = setup_single_asset_coins(
        wallet.address(),
        asset_id,
        number_of_coins,
        DEFAULT_COIN_AMOUNT,
    );

    let config = Config {
        utxo_validation: false,
        addr: defaults::FUEL_NODE_ADDR.parse().unwrap(),
        ..Config::local_node()
    };

    let (client, _) = setup_test_client(coins, vec![], Some(config), None, None).await;

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    let contract_id = Contract::deploy(
        bin_path_str,
        &wallet,
        tx_params(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let contract_id = contract_id.to_string();

    info!("Contract deployed at: {}", &contract_id);

    Ok(())
}

pub mod test_web {

    use super::{tx_params, FuelIndexerTest, WORKSPACE_ROOT};
    use crate::defaults;
    use actix_service::ServiceFactory;
    use actix_web::{
        body::MessageBody,
        dev::{ServiceRequest, ServiceResponse},
        web, App, Error, HttpResponse, HttpServer, Responder,
    };
    use async_std::sync::Arc;
    use fuels::{
        prelude::{CallParameters, Contract, Provider, WalletUnlocked},
        signers::Signer,
    };
    use fuels_core::parameters::StorageConfiguration;
    use std::path::Path;
    use tracing::info;

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

    async fn fuel_indexer_test_transfer(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let call_params = CallParameters::new(Some(1_000_000), None, None);

        let _ = state
            .contract
            .methods()
            .trigger_transfer()
            .append_variable_outputs(1)
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

    async fn fuel_indexer_test_logdata(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
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
        let _ = state
            .contract
            .methods()
            .trigger_transferout()
            .tx_params(tx_params())
            .call()
            .await;

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_messageout(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
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

    pub fn app(
        contract: FuelIndexerTest,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = Error,
        >,
    > {
        let state = web::Data::new(Arc::new(AppState { contract }));

        App::new()
            .app_data(state)
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
    }

    pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
        let wallet_path = Path::new(WORKSPACE_ROOT).join("assets").join("wallet.json");
        let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();
        let mut wallet = WalletUnlocked::load_keystore(
            wallet_path_str,
            defaults::WALLET_PASSWORD,
            None,
        )
        .unwrap();

        let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();

        wallet.set_provider(provider.clone());

        info!(
            "Wallet({}) keystore at: {}",
            wallet.address(),
            wallet_path.display()
        );

        let bin_path = Path::new(WORKSPACE_ROOT)
            .join("contracts")
            .join("fuel-indexer-test")
            .join("out")
            .join("debug")
            .join("fuel-indexer-test.bin");

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

        info!("Using contract at {}", contract_id.to_string());

        info!("Starting server at {}", defaults::WEB_API_ADDR);

        let _ = HttpServer::new(move || {
            app(FuelIndexerTest::new(contract_id.clone(), wallet.clone()))
        })
        .bind(defaults::WEB_API_ADDR)
        .unwrap()
        .run()
        .await;

        Ok(())
    }
}

pub async fn indexer_service(
    manifest: Manifest,
    config: Option<IndexerConfig>,
) -> IndexerService {
    let config = config.unwrap_or_else(|| IndexerConfig {
        fuel_node: FuelNodeConfig::from(
            defaults::FUEL_NODE_ADDR
                .parse::<std::net::SocketAddr>()
                .unwrap(),
        ),
        database: DatabaseConfig::Postgres {
            user: "postgres".into(),
            password: "my-secret".into(),
            host: "127.0.0.1".into(),
            port: "5432".into(),
            database: "postgres".to_string(),
        },
        graphql_api: GraphQLConfig::default(),
        metrics: false,
    });

    let mut indexer_service = IndexerService::new(config, None).await.unwrap();

    indexer_service
        .register_indices(Some(manifest), true)
        .await
        .expect("Failed to initialize indexer.");

    indexer_service
}
