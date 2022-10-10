extern crate alloc;
use fuel_indexer::IndexerService;
use fuel_indexer_lib::config::{
    DatabaseConfig, FuelNodeConfig, GraphQLConfig, IndexerConfig,
};
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{defaults, fixtures::tx_params};
use fuels::prelude::{
    setup_single_asset_coins, setup_test_client, AssetId, Contract, Provider,
    WalletUnlocked, DEFAULT_COIN_AMOUNT,
};
use fuels::signers::Signer;
use fuels_abigen_macro::abigen;
use fuels_core::parameters::StorageConfiguration;
use std::path::Path;

const MANIFEST: &str = include_str!("./../assets/simple_wasm.yaml");
const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

abigen!(
    Simple,
    "examples/simple-wasm/contracts/out/debug/contracts-abi.json"
);

#[tokio::test]
#[cfg_attr(feature = "e2e", ignore)]
async fn test_can_trigger_event_from_contract_and_index_emited_event_in_postgres() {
    let workdir = Path::new(WORKSPACE_DIR);

    let wallet_path = workdir.join("assets/wallet.json");
    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

    let mut wallet =
        WalletUnlocked::load_keystore(&wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let bin_path =
        workdir.join("../examples/simple-wasm/contracts/out/debug/contracts.bin");
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

    let (client, _) = setup_test_client(coins, vec![], None, None).await;

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

    let contract = SimpleBuilder::new(contract_id.to_string(), wallet);

    let _ = contract.gimme_someevent(78).call().await;
    let _ = contract.gimme_anotherevent(899).call().await;

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
    };

    let mut indexer_service = IndexerService::new(config, None).await.unwrap();

    let manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file");

    indexer_service
        .register_indices(Some(manifest), true)
        .await
        .expect("Failed to initialize indexer");

    indexer_service.run().await;
}
