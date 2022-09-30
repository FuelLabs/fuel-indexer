extern crate alloc;
use fuel_indexer::{
    config::{DatabaseConfig, FuelNodeConfig, GraphQLConfig, IndexerConfig},
    IndexerService, Manifest,
};
use fuels::node::{
    chain_config::{ChainConfig, StateConfig},
    service::DbType,
};
use fuels::prelude::{
    setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, LocalWallet,
    Provider, TxParameters, DEFAULT_COIN_AMOUNT,
};
use fuels::signers::Signer;
use fuels_abigen_macro::abigen;
use std::path::Path;

const MANIFEST: &str = include_str!("./../assets/simple_wasm.yaml");
const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");
const FUEL_NODE_ADDR: &str = "0.0.0.0:4000";

abigen!(
    Simple,
    "examples/simple-wasm/contracts/out/debug/contracts-abi.json"
);

fn tx_params() -> TxParameters {
    let gas_price = 0;
    let gas_limit = 1_000_000;
    let byte_price = 0;
    TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price), None)
}

#[tokio::test]
#[cfg_attr(feature = "e2e", ignore)]
async fn test_can_trigger_event_from_contract_and_index_emited_event_in_postgres() {
    let workdir = Path::new(WORKSPACE_DIR);

    let p = workdir.join("components/web/wallet.json");
    let path = p.as_os_str().to_str().unwrap();
    let mut wallet =
        LocalWallet::load_keystore(&path, "password", None).expect("Could load keys");

    let p = workdir.join("../examples/simple-wasm/contracts/out/debug/contracts.bin");
    let path = p.as_os_str().to_str().unwrap();
    let _compiled = Contract::load_sway_contract(path).unwrap();

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
        addr: FUEL_NODE_ADDR.parse().unwrap(),
        ..Config::local_node()
    };

    let (client, _) = setup_test_client(coins, config).await;

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    let contract_id = Contract::deploy(path, &wallet, tx_params()).await.unwrap();

    let contract: Simple = Simple::new(contract_id.to_string(), wallet);
    let _ = contract.gimme_someevent(78).call().await;
    let _ = contract.gimme_anotherevent(899).call().await;

    let config = IndexerConfig {
        fuel_node: FuelNodeConfig::from(
            FUEL_NODE_ADDR.parse::<std::net::SocketAddr>().unwrap(),
        ),
        database: DatabaseConfig::Postgres {
            user: "postgres".into(),
            password: "my-secret".into(),
            host: "127.0.0.1".into(),
            port: "5432".into(),
            database: "postgres".to_string(),
        },
        graphql_api: GraphQLConfig::default(),
    };

    let mut indexer_service = IndexerService::new(config).await.unwrap();

    let manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file");

    indexer_service
        .register_indices(Some(manifest), true)
        .await
        .expect("Failed to initialize indexer");

    indexer_service.run().await;
}
