extern crate alloc;
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{defaults, fixtures::indexer_service_postgres};
use fuels::prelude::{LoadConfiguration, TxParameters};
use fuels::{
    macros::abigen,
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Contract, Provider,
        WalletUnlocked, DEFAULT_COIN_AMOUNT,
    },
};
use std::path::Path;

const SIMPLE_WASM_MANIFEST: &str =
    include_str!("./../../components/indices/simple-wasm/simple_wasm.yaml");
const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

abigen!(Contract(
    name = "Simple",
    abi =
        "packages/fuel-indexer-tests/contracts/simple-wasm/out/debug/contracts-abi.json"
));

#[tokio::test]
#[cfg_attr(feature = "e2e", ignore)]
async fn test_can_trigger_event_from_contract_and_index_emited_event_in_postgres() {
    let workdir = Path::new(WORKSPACE_DIR);

    let wallet_path = workdir.join("test-chain-config.json");
    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();

    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let bin_path = workdir.join("contracts/simple-wasm/out/debug/contracts.bin");
    let bin_path_str = bin_path.as_os_str().to_str().unwrap();
    let loaded_contract =
        Contract::load_from(bin_path_str, LoadConfiguration::default()).unwrap();

    let number_of_coins = 11;
    let asset_id = AssetId::zeroed();
    let coins = setup_single_asset_coins(
        wallet.address(),
        asset_id,
        number_of_coins,
        DEFAULT_COIN_AMOUNT,
    );

    let (client, _) = setup_test_client(coins, vec![], None, None, None).await;

    let provider = Provider::new(client);

    wallet.set_provider(provider.clone());

    let contract_id = loaded_contract
        .deploy(&wallet, TxParameters::default())
        .await
        .unwrap();

    let contract = Simple::new(contract_id, wallet);

    let _ = contract.methods().gimme_someevent(78).call().await;
    let _ = contract.methods().gimme_anotherevent(899).call().await;

    let mut srvc = indexer_service_postgres(None, None).await;

    let manifest: Manifest =
        serde_yaml::from_str(SIMPLE_WASM_MANIFEST).expect("Bad yaml file.");

    srvc.register_indexer_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;
}
