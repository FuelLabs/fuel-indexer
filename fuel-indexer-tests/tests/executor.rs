use fuel_indexer::{
    executor::WasmIndexExecutor, Executor, IndexerError, Manifest, Module, SchemaManager,
};
use fuel_indexer_schema::types::{
    fuel::{BlockData, TransactionData},
    transaction::TransactionStatus,
};
use fuel_indexer_tests::assets::{BAD_MANIFEST, BAD_WASM_BYTES, MANIFEST, WASM_BYTES};
use fuel_tx::{Receipt, Transaction};
use fuels_abigen_macro::abigen;
use fuels_core::{abi_encoder::ABIEncoder, types::Bits256, Tokenizable};
use sqlx::{Connection, Row};
use std::path::Path;

pub const MANIFEST_ROOT: &str = env!("CARGO_MANIFEST_DIR");

abigen!(
    MyContract,
    "fuel-indexer-tests/contracts/simple-wasm/out/debug/contracts-abi.json"
);

#[derive(Debug)]
struct Thing1 {
    id: i64,
    account: String,
}

#[tokio::test]
async fn test_can_create_wasm_executor_and_index_abi_entity_in_postgres() {
    let database_url = "postgres://postgres:my-secret@127.0.0.1:5432";

    create_wasm_executor_and_handle_events(database_url).await;

    let mut conn = sqlx::PgConnection::connect(database_url)
        .await
        .expect("Database connection failed.");

    let row =
        sqlx::query("select id,account from test_namespace.thing1 where id = 1020;")
            .fetch_one(&mut conn)
            .await
            .expect("Database query failed");

    let id = row.get(0);
    let account = row.get(1);

    let data = Thing1 { id, account };

    assert_eq!(data.id, 1020);
    assert_eq!(
        data.account,
        "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
    );
}

// NOTE: This is a hack to update the `manifest` with the proper absolute path.
// This is already done in the #[indexer] attribute, but since these tests skip
// over that functionality we have to hack that in here. This allows us to keep
// a single consistent manifest file, regardless of where/how it's used.
fn update_manifest_with_proper_paths(manifest: &mut Manifest) {
    let manifest_dir = Path::new(MANIFEST_ROOT);
    manifest.graphql_schema = manifest_dir
        .join("..")
        .join(&manifest.graphql_schema)
        .into_os_string()
        .to_str()
        .unwrap()
        .to_string();
    manifest.abi = Some(
        manifest_dir
            .join("..")
            .join(&manifest.abi.clone().unwrap())
            .into_os_string()
            .to_str()
            .unwrap()
            .to_string(),
    );
    manifest.module = Module::Wasm(
        manifest_dir
            .join("..")
            .join(&manifest.graphql_schema)
            .into_os_string()
            .to_str()
            .unwrap()
            .to_string(),
    );
}

#[tokio::test]
async fn test_can_create_wasm_executor_and_index_abi_entity_in_sqlite() {
    let database_url = format!("sqlite://{}/test.db", MANIFEST_ROOT);

    create_wasm_executor_and_handle_events(&database_url).await;

    let mut conn = sqlx::SqliteConnection::connect(&database_url)
        .await
        .expect("Database connection failed.");

    let row = sqlx::query("select id,account from thing1 where id = 1020;")
        .fetch_one(&mut conn)
        .await
        .expect("Database query failed");

    let id = row.get(0);
    let account = row.get(1);

    let data = Thing1 { id, account };

    assert_eq!(data.id, 1020);
    assert_eq!(
        data.account,
        "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
    );
}

async fn create_wasm_executor_and_handle_events(database_url: &str) {
    let mut manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file.");
    update_manifest_with_proper_paths(&mut manifest);

    let sm = SchemaManager::new(database_url).await.unwrap();
    let _ = sm.new_schema(&manifest.namespace, &manifest.graphql_schema().unwrap());

    let mut bad_manifest: Manifest =
        serde_yaml::from_str(BAD_MANIFEST).expect("Bad yaml file.");
    update_manifest_with_proper_paths(&mut bad_manifest);

    let executor =
        WasmIndexExecutor::new(database_url.to_string(), bad_manifest, BAD_WASM_BYTES)
            .await;

    match executor {
        Err(IndexerError::MissingHandler) => (),
        e => panic!("Expected missing handler error {:#?}", e),
    }

    let executor =
        WasmIndexExecutor::new(database_url.to_string(), manifest, WASM_BYTES).await;
    assert!(executor.is_ok());

    let mut executor = executor.unwrap();

    let evt1 = SomeEvent {
        id: 1020,
        account: Bits256([0xaf; 32]),
    };
    let evt2 = AnotherEvent {
        id: 100,
        account: Bits256([0x5a; 32]),
        hash: Bits256([0x43; 32]),
    };

    let some_event =
        ABIEncoder::encode(&[evt1.into_token()]).expect("Failed compile test");
    let another_event =
        ABIEncoder::encode(&[evt2.into_token()]).expect("Failed compile test");

    let result = executor
        .handle_events(vec![BlockData {
            id: [0u8; 32].into(),
            producer: [0u8; 32].into(),
            time: 1,
            height: 0,
            transactions: vec![
                TransactionData {
                    id: [0u8; 32].into(),
                    status: TransactionStatus::default(),
                    receipts: vec![
                        Receipt::Call {
                            id: [0u8; 32].into(),
                            to: [0u8; 32].into(),
                            amount: 400,
                            asset_id: [0u8; 32].into(),
                            gas: 4,
                            param1: 2048508220,
                            param2: 0,
                            pc: 0,
                            is: 0,
                        },
                        Receipt::ReturnData {
                            id: [0u8; 32].into(),
                            ptr: 2342143,
                            len: some_event.resolve(0).len() as u64,
                            digest: [0u8; 32].into(),
                            data: some_event.resolve(0),
                            pc: 0,
                            is: 0,
                        },
                    ],
                    transaction: Transaction::default(),
                },
                TransactionData {
                    id: [0u8; 32].into(),
                    status: TransactionStatus::default(),
                    receipts: vec![
                        Receipt::Call {
                            id: [0u8; 32].into(),
                            to: [0u8; 32].into(),
                            amount: 400,
                            asset_id: [0u8; 32].into(),
                            gas: 4,
                            param1: 2379805026,
                            param2: 0,
                            pc: 0,
                            is: 0,
                        },
                        Receipt::ReturnData {
                            id: [0u8; 32].into(),
                            ptr: 2342143,
                            len: another_event.resolve(0).len() as u64,
                            digest: [0u8; 32].into(),
                            data: another_event.resolve(0),
                            pc: 0,
                            is: 0,
                        },
                    ],
                    transaction: Transaction::default(),
                },
            ],
        }])
        .await;
    assert!(result.is_ok());
}
