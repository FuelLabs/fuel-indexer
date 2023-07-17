extern crate alloc;
use fuel_indexer::{Executor, IndexerConfig, WasmIndexExecutor};
use fuel_indexer_lib::{config::DatabaseConfig, manifest::Manifest};
use fuel_indexer_tests::{
    defaults,
    fixtures::{indexer_service_postgres, TestPostgresDb},
};
use fuels::prelude::{LoadConfiguration, TxParameters};
use fuels::{
    accounts::wallet::WalletUnlocked,
    macros::abigen,
    prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Contract, Provider,
        DEFAULT_COIN_AMOUNT,
    },
};
use std::path::Path;
use std::str::FromStr;

const SIMPLE_WASM_MANIFEST: &str =
    include_str!("./../../components/indices/simple-wasm/simple_wasm.yaml");
const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

abigen!(Contract(
    name = "Simple",
    abi =
        "packages/fuel-indexer-tests/contracts/simple-wasm/out/debug/contracts-abi.json"
));

#[tokio::test]
async fn test_wasm_executor_can_meter_execution() {
    use async_std::{fs::File, io::ReadExt};

    if let Ok(current_dir) = std::env::current_dir() {
        println!("Current directory: {}", current_dir.display());
    }

    use std::env;

    if let Ok(mut current_dir) = env::current_dir() {
        current_dir.pop();
        current_dir.pop();
        if let Err(e) = env::set_current_dir(current_dir) {
            eprintln!("Failed to change directory: {}", e);
        }
    }

    let manifest = Manifest::from_file(
        "packages/fuel-indexer-tests/components/indices/simple-wasm/simple_wasm.yaml",
    )
    .unwrap();

    match &manifest.module {
        fuel_indexer_lib::manifest::Module::Wasm(ref module) => {
            let mut bytes = Vec::<u8>::new();
            let mut file = File::open(module).await.unwrap();
            file.read_to_end(&mut bytes).await.unwrap();

            let test_db = TestPostgresDb::new().await.unwrap();
            let pool = fuel_indexer_database::IndexerConnectionPool::Postgres(
                test_db.pool.clone(),
            );
            let mut config = IndexerConfig::default();
            config.database = DatabaseConfig::from_str(&test_db.url).unwrap();
            // not enough points to finish execution
            config.metering_points = Some(100u64);

            let mut executor =
                WasmIndexExecutor::new(&config, &manifest, bytes.clone(), pool)
                    .await
                    .unwrap();

            let blocks: Vec<fuel_indexer_types::fuel::BlockData> = vec![];

            if let Err(e) = executor.handle_events(blocks.clone()).await {
                if let fuel_indexer::IndexerError::RuntimeError(e) = e {
                    if let Some(e) = e.to_trap() {
                        assert_eq!(e, wasmer_types::TrapCode::UnreachableCodeReached);
                        assert!(executor.metering_points_exhausted().await);
                        println!("Metering points exhausted.");
                    } else {
                        panic!("Expected exhausted metering points");
                    }
                } else {
                    match executor.get_remaining_metering_points().await {
                        Some(wasmer_middlewares::metering::MeteringPoints::Remaining(pts)) => {
                            assert!(pts > 0)
                        }
                        _ => panic!("Expected remaining points > 0"),
                    }
                }
            }
        }
        _ => panic!("Expected a WASM module in the manifest but got a Native module instead."),
    }
}

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
