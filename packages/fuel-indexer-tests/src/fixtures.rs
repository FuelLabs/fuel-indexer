use crate::{defaults, WORKSPACE_ROOT};
use axum::Router;
use fuel_indexer::IndexerService;
use fuel_indexer_api_server::api::GraphQlApi;
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::config::{
    DatabaseConfig, FuelNodeConfig, GraphQLConfig, IndexerConfig,
};
use fuels::core::parameters::StorageConfiguration;
use fuels::{
    fuels_abigen::abigen,
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Bech32ContractId, Config,
        Contract, Provider, TxParameters, WalletUnlocked, DEFAULT_COIN_AMOUNT,
    },
    signers::Signer,
};
use sqlx::{
    pool::{Pool, PoolConnection},
    Postgres,
};
use std::path::Path;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    FuelIndexerTest,
    "packages/fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json"
);

pub async fn postgres_connection_pool() -> Pool<Postgres> {
    let config = DatabaseConfig::Postgres {
        user: "postgres".into(),
        password: "my-secret".into(),
        host: "127.0.0.1".into(),
        port: "5432".into(),
        database: "postgres".to_string(),
    };
    match IndexerConnectionPool::connect(&config.to_string())
        .await
        .unwrap()
    {
        IndexerConnectionPool::Postgres(p) => p,
    }
}

pub async fn postgres_connection() -> PoolConnection<Postgres> {
    let config = DatabaseConfig::Postgres {
        user: "postgres".into(),
        password: "my-secret".into(),
        host: "127.0.0.1".into(),
        port: "5432".into(),
        database: "postgres".to_string(),
    };
    match IndexerConnectionPool::connect(&config.to_string())
        .await
        .unwrap()
    {
        IndexerConnectionPool::Postgres(p) => p.acquire().await.unwrap(),
    }
}

pub fn http_client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap()
}

pub fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
}

pub async fn setup_test_fuel_node(
    wallet_path: &str,
    contract_bin_path: &str,
    host: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => {
            EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
        }
        None => EnvFilter::new("info"),
    };

    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init();

    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let _compiled = Contract::load_contract(contract_bin_path, &None).unwrap();

    let number_of_coins = defaults::COIN_AMOUNT;
    let asset_id = AssetId::zeroed();
    let coins = setup_single_asset_coins(
        wallet.address(),
        asset_id,
        number_of_coins,
        DEFAULT_COIN_AMOUNT,
    );

    let config = Config {
        utxo_validation: false,
        addr: host.parse().unwrap(),
        ..Config::local_node()
    };

    let (client, _) = setup_test_client(coins, vec![], Some(config), None, None).await;

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    let contract_id = Contract::deploy(
        contract_bin_path,
        &wallet,
        tx_params(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let contract_id = contract_id.to_string();

    println!("Contract deployed at: {}", &contract_id);

    Ok(contract_id)
}

pub async fn get_contract_id(
    wallet_path: &str,
    contract_bin_path: &str,
) -> Result<(WalletUnlocked, Bech32ContractId), Box<dyn std::error::Error>> {
    get_contract_id_with_host(
        wallet_path,
        contract_bin_path,
        defaults::FUEL_NODE_ADDR.to_string(),
    )
    .await
}

pub async fn get_contract_id_with_host(
    wallet_path: &str,
    contract_bin_path: &str,
    host: String,
) -> Result<(WalletUnlocked, Bech32ContractId), Box<dyn std::error::Error>> {
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => {
            EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
        }
        None => EnvFilter::new("info"),
    };

    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init();

    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let provider = Provider::connect(&host).await.unwrap();

    wallet.set_provider(provider.clone());

    let _compiled = Contract::load_contract(contract_bin_path, &None).unwrap();

    let contract_id = Contract::deploy(
        contract_bin_path,
        &wallet,
        tx_params(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    println!("Using contract at {:?}", &contract_id);

    Ok((wallet, contract_id))
}

pub async fn api_server_app_postgres() -> Router {
    let config = IndexerConfig {
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
        metrics: true,
        stop_idle_indexers: true,
    };

    let pool = IndexerConnectionPool::connect(&config.database.to_string())
        .await
        .expect("Failed to create connection pool");

    GraphQlApi::build(config, pool, None).await.unwrap()
}

pub async fn indexer_service_postgres() -> IndexerService {
    let config = IndexerConfig {
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
        stop_idle_indexers: true,
    };

    let pool = IndexerConnectionPool::connect(&config.database.to_string())
        .await
        .expect("Failed to create connection pool");

    IndexerService::new(config, pool, None).await.unwrap()
}

pub async fn connect_to_deployed_contract(
) -> Result<FuelIndexerTest, Box<dyn std::error::Error>> {
    let wallet_path = Path::new(WORKSPACE_ROOT)
        .join("assets")
        .join("test-chain-config.json");
    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();
    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();

    wallet.set_provider(provider.clone());

    println!(
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

    println!("Using contract at {contract_id}",);

    let contract = FuelIndexerTest::new(contract_id, wallet.clone());

    Ok(contract)
}

pub mod test_web {

    use super::{get_contract_id, tx_params, FuelIndexerTest};
    use crate::defaults;
    use actix_service::ServiceFactory;
    use actix_web::{
        body::MessageBody,
        dev::{ServiceRequest, ServiceResponse},
        web, App, Error, HttpResponse, HttpServer, Responder,
    };
    use async_std::sync::Arc;
    use fuels::prelude::{CallParameters, Provider};
    use std::path::Path;

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

    async fn fuel_indexer_test_callreturn(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_callreturn()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_multiargs(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_multiargs()
            .tx_params(tx_params())
            .call()
            .await
            .unwrap();

        HttpResponse::Ok()
    }

    async fn fuel_indexer_test_optional_schema_fields(
        state: web::Data<Arc<AppState>>,
    ) -> impl Responder {
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

    async fn fuel_indexer_test_get_block_height() -> impl Responder {
        let provider = Provider::connect(defaults::FUEL_NODE_ADDR).await.unwrap();
        let block_height = provider.latest_block_height().await.unwrap();

        HttpResponse::Ok().body(block_height.to_string())
    }

    async fn fuel_indexer_test_tuple(state: web::Data<Arc<AppState>>) -> impl Responder {
        let _ = state
            .contract
            .methods()
            .trigger_tuple()
            .tx_params(tx_params())
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
            .route("/callreturn", web::post().to(fuel_indexer_test_callreturn))
            .route("/multiarg", web::post().to(fuel_indexer_test_multiargs))
            .route(
                "/optionals",
                web::post().to(fuel_indexer_test_optional_schema_fields),
            )
            .route(
                "/block_height",
                web::get().to(fuel_indexer_test_get_block_height),
            )
            .route("/tuples", web::post().to(fuel_indexer_test_tuple))
    }

    pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
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

        println!("Starting server at {}", defaults::WEB_API_ADDR);

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
