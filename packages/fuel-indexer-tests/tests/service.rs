extern crate alloc;
use fuel_indexer::prelude::fuel::{BlockData, Consensus, Header};
use fuel_indexer::{Executor, IndexerConfig, WasmIndexExecutor};
use fuel_indexer_lib::WasmIndexerError;
use fuel_indexer_lib::{config::DatabaseConfig, manifest::Manifest};
use fuel_indexer_tests::fixtures::TestPostgresDb;
use fuel_types::Bytes32;
use std::str::FromStr;
use std::sync::{atomic::AtomicBool, Arc};

#[tokio::test]
async fn test_wasm_executor_can_meter_execution() {
    use async_std::{fs::File, io::ReadExt};

    if let Ok(mut current_dir) = std::env::current_dir() {
        if current_dir.ends_with("fuel-indexer-tests") {
            current_dir.pop();
            current_dir.pop();
        }

        if let Err(e) = std::env::set_current_dir(current_dir) {
            eprintln!("Failed to change directory: {}", e);
        }
    }

    let manifest = Manifest::from_file(
        "packages/fuel-indexer-tests/indexers/fuel-indexer-test/fuel_indexer_test.yaml",
    )
    .unwrap();

    match &manifest.module() {
        fuel_indexer_lib::manifest::Module::Wasm(ref module) => {
            let mut bytes = Vec::<u8>::new();
            let mut file = File::open(module).await.unwrap();
            file.read_to_end(&mut bytes).await.unwrap();

            let test_db = TestPostgresDb::new().await.unwrap();
            let pool = fuel_indexer_database::IndexerConnectionPool::Postgres(
                test_db.pool.clone(),
            );
            let config = IndexerConfig {
                database: DatabaseConfig::from_str(&test_db.url).unwrap(),
                // not enough points to finish execution
                metering_points: Some(100u64),
                ..Default::default()
            };

            let schema_version = manifest
                .graphql_schema_content()
                .unwrap()
                .version()
                .to_string();

            let kill_switch = Arc::new(AtomicBool::new(false));
            let mut executor = WasmIndexExecutor::new(
                &config,
                &manifest,
                bytes.clone(),
                pool,
                schema_version,
                kill_switch,
            )
            .await
            .unwrap();

            let kill_switch = std::sync::Arc::new(AtomicBool::new(false));

            let blocks: Vec<fuel_indexer_types::fuel::BlockData> = vec![];

            if let Err(e) = executor.handle_events(kill_switch, blocks.clone()).await {
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
                        Some(
                            wasmer_middlewares::metering::MeteringPoints::Remaining(pts),
                        ) => {
                            assert!(pts > 0)
                        }
                        _ => panic!("Expected remaining points > 0"),
                    }
                }
            }
        }
        _ => panic!(
            "Expected a WASM module in the manifest but got a Native module instead."
        ),
    }
}

#[tokio::test]
async fn test_wasm_executor_exit_codes() {
    use async_std::{fs::File, io::ReadExt};

    if let Ok(mut current_dir) = std::env::current_dir() {
        if current_dir.ends_with("fuel-indexer-tests") {
            current_dir.pop();
            current_dir.pop();
        }

        if let Err(e) = std::env::set_current_dir(current_dir) {
            eprintln!("Failed to change directory: {}", e);
        }
    }

    let manifest = Manifest::from_file(
        "packages/fuel-indexer-tests/indexers/fuel-indexer-test/fuel_indexer_test.yaml",
    )
    .unwrap();

    match &manifest.module() {
        fuel_indexer_lib::manifest::Module::Wasm(ref module) => {
            let mut bytes = Vec::<u8>::new();
            let mut file = File::open(module).await.unwrap();
            file.read_to_end(&mut bytes).await.unwrap();

            let test_db = TestPostgresDb::new().await.unwrap();
            let pool = fuel_indexer_database::IndexerConnectionPool::Postgres(
                test_db.pool.clone(),
            );
            let config = IndexerConfig::default();

            let schema_version = manifest
                .graphql_schema_content()
                .unwrap()
                .version()
                .to_string();

            let kill_switch = Arc::new(AtomicBool::new(false));
            let mut executor = WasmIndexExecutor::new(
                &config,
                &manifest,
                bytes.clone(),
                pool,
                schema_version,
                kill_switch.clone(),
            )
            .await
            .unwrap();

            let block_1_header = Header {
                id: Bytes32::zeroed(),
                da_height: 1,
                transactions_count: 0,
                message_receipt_count: 0,
                transactions_root: Bytes32::zeroed(),
                message_receipt_root: Bytes32::zeroed(),
                height: 1,
                prev_root: Bytes32::zeroed(),
                time: 0,
                application_hash: Bytes32::zeroed(),
            };

            let block_1 = BlockData {
                height: 1,
                id: Bytes32::zeroed(),
                header: block_1_header,
                producer: None,
                time: 0,
                consensus: Consensus::Unknown,
                transactions: vec![],
            };

            let blocks: Vec<fuel_indexer_types::fuel::BlockData> = vec![block_1];

            // Test 1

            // Since we are only starting the executor, and not the whole Fuel
            // Indexer Servoce, the database tables are not initialized, and any
            // database operation performed by the executor will fail.
            if let Err(e) = executor
                .handle_events(kill_switch.clone(), blocks.clone())
                .await
            {
                if let fuel_indexer::IndexerError::RuntimeError(e) = e {
                    match e.downcast::<WasmIndexerError>() {
                        Ok(err_code) => {
                            assert_eq!(err_code, WasmIndexerError::DatabaseError);
                        }
                        Err(e) => {
                            panic!("Expected a WASM exit code but got: {e:?}");
                        }
                    }
                } else {
                    panic!("Expected a RuntimeError but got: {e:?}");
                }
            }

            // Test 2

            // Trigger kill switch.

            kill_switch.store(true, std::sync::atomic::Ordering::SeqCst);

            if let Err(e) = executor.handle_events(kill_switch, blocks.clone()).await {
                if let fuel_indexer::IndexerError::RuntimeError(e) = e {
                    match e.downcast::<WasmIndexerError>() {
                        Ok(err_code) => {
                            assert_eq!(err_code, WasmIndexerError::KillSwitch);
                        }
                        Err(e) => {
                            panic!("Expected a WASM exit code but got: {e:?}");
                        }
                    }
                } else {
                    panic!("Expected a RuntimeError but got: {e:?}");
                }
            }
        }
        _ => panic!(
            "Expected a WASM module in the manifest but got a Native module instead."
        ),
    }
}
