use actix_service::Service;
use actix_web::test;
use fuel_indexer::IndexerService;
use fuel_indexer_database::{queries, IndexerConnection};
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{
    assets, defaults,
    fixtures::{
        connect_to_deployed_contract, indexer_service_postgres, postgres_connection,
        postgres_connection_pool, setup_example_test_fuel_node, test_web::app,
    },
    utils::update_test_manifest_asset_paths,
    WORKSPACE_ROOT,
};
use fuel_indexer_types::{Address, ContractId, Identity};
use fuels::signers::WalletUnlocked;
use hex::FromHex;
use lazy_static::lazy_static;
use sqlx::{
    pool::{Pool, PoolConnection},
    types::BigDecimal,
    Postgres, Row,
};
use std::{path::Path, str::FromStr};
use tokio::time::{sleep, Duration};

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_events_with_multiple_args_in_index_handler_postgres()
{
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/multiarg").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let block_row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.block ORDER BY height DESC LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let height: i64 = block_row.get(1);
    let timestamp: i64 = block_row.get(2);
    assert!(height >= 1);
    assert!(timestamp > 0);

    let ping_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pingentity WHERE id = 12345")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let ping_value: i64 = ping_row.get(1);
    assert_eq!(ping_value, 12345);

    let pong_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pongentity WHERE id = 45678")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let pong_value: i64 = pong_row.get(1);
    assert_eq!(pong_value, 45678);

    let pung_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 123")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let pung_from: String = pung_row.get(3);
    let from_buff = ContractId::from_str(&pung_from).unwrap();

    let contract_buff = ContractId::from_str(
        "0x322ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(
        Identity::ContractId(from_buff),
        Identity::ContractId(contract_buff),
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_callreturn_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/callreturn").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 3")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let value: i64 = row.get(1);
    let is_pung: bool = row.get(2);
    let pung_from: String = row.get(3);
    println!("{}", &pung_from);
    let from_buff = Address::from_str(&pung_from).unwrap();

    let addr_buff = Address::from_str(
        "0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(value, 12345);
    assert!(is_pung);
    assert_eq!(Identity::Address(from_buff), Identity::Address(addr_buff),);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_blocks_and_transactions_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;

    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/block").to_request();
    let _ = app.call(req).await;
    fuel_node_handle.abort();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let mut conn = pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.block WHERE height = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let id: i64 = row.get(0);
    let height: i64 = row.get(1);
    let timestamp: i64 = row.get(2);

    assert_eq!(height, 1);
    assert!(timestamp > 0);

    let row = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test_index1.tx WHERE block = {id}",
    ))
    .fetch_all(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.len(), 2);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_ping_event_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pingentity WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let id: i64 = row.get(0);
    let value: i64 = row.get(1);

    assert_eq!(id, 1);
    assert_eq!(value, 123);

    // Ping also triggers the 128-bit integer test as well
    let mut conn = pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.u16entity WHERE id = 9999")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let id: i64 = row.get(0);
    let value1: BigDecimal = row.get(1);
    let value2: BigDecimal = row.get(2);

    assert_eq!(
        value1,
        BigDecimal::from_str("340282366920938463463374607431768211454").unwrap()
    );
    assert_eq!(
        value2,
        BigDecimal::from_str("170141183460469231731687303715884105727").unwrap()
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_transfer_event_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/transfer").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.transfer LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let amount: i64 = row.get(3);
    let asset_id: &str = row.get(4);

    assert_eq!(amount, 1); // value is defined in test contract
    assert_eq!(asset_id, defaults::TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres", feature = "pg-embed-skip"))]
async fn test_can_trigger_and_index_log_event_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let mut conn = pool.acquire().await.unwrap();
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/log").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.log WHERE ra = 8675309 LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let ra: i64 = row.get(2);

    assert_eq!(ra, 8675309);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres", feature = "pg-embed-skip"))]
async fn test_can_trigger_and_index_logdata_event_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/logdata").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let value: i64 = row.get(1);
    let is_pung: bool = row.get(2);
    let pung_from: String = row.get(3);
    let from_buff = Address::from_str(&pung_from).unwrap();

    let addr_buff = Address::from_str(
        "0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(value, 456);
    assert!(is_pung);
    assert_eq!(Identity::Address(from_buff), Identity::Address(addr_buff));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_scriptresult_event_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/scriptresult").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.scriptresult LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let result: i64 = row.get(1);
    let gas_used: i64 = row.get(2);
    let data: String = row.get(3);

    let expected = hex::decode(data)
        .unwrap()
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");

    assert!((0..=1).contains(&result));
    assert!(gas_used > 0);
    assert_eq!(expected, "1,1,1,1,1".to_string());
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres", feature = "pg-embed-skip"))]
async fn test_can_trigger_and_index_transferout_event_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/transferout").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.transferout LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let recipient: &str = row.get(2);
    let amount: i64 = row.get(3);
    let asset_id: &str = row.get(4);

    assert_eq!(
        recipient,
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(amount, 1);
    assert_eq!(asset_id, defaults::TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres", feature = "pg-embed-skip"))]
async fn test_can_trigger_and_index_messageout_event_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/messageout").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.messageout LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let message_id: i64 = row.get(0);
    let recipient: &str = row.get(2);
    let amount: i64 = row.get(3);
    let len: i64 = row.get(5);

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.messageentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let example_message_id: i64 = row.get(0);
    let message: &str = row.get(1);

    assert!((message_id > 0 && message_id < i64::MAX));
    assert_eq!(
        recipient,
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(amount, 100);

    assert_eq!(example_message_id, 1234);
    assert_eq!(message, "abcdefghijklmnopqrstuvwxyz123456");
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres", feature = "pg-embed-skip"))]
async fn test_can_index_event_with_optional_fields_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/optionals").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.optionentity WHERE id = 8675309",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let id: i64 = row.get(0);
    let req_int: i64 = row.get(1);
    let opt_int_some: Option<i64> = row.get(2);
    let opt_addr_none: Option<&str> = row.get(3);

    assert_eq!(id, 8675309);
    assert_eq!(req_int, 100);
    assert!(opt_int_some.is_some());

    let opt_int = opt_int_some.unwrap();
    assert_eq!(opt_int, 999);

    assert!(opt_addr_none.is_none());
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_index_metadata_is_saved_when_indexer_macro_is_called_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.indexmetadataentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();
    let block_height: i64 = row.get(0);
    let time: i64 = row.get(1);

    assert!(block_height >= 1);
    assert!(time >= 1);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_index_respects_start_block_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut conn = pool.acquire().await.unwrap();

    let result = sqlx::query("DELETE FROM fuel_indexer_test_index1.tx")
        .execute(&mut conn)
        .await
        .unwrap();
    let result = sqlx::query("DELETE FROM fuel_indexer_test_index1.block")
        .execute(&mut conn)
        .await
        .unwrap();

    let mut srvc = indexer_service_postgres().await;

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::get().uri("/block_height").to_request();
    let res = test::call_and_read_body(&app, req).await;
    let block_height = String::from_utf8(res.to_vec())
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    manifest.start_block = Some(block_height + 2);

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let pre_check = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test_index1.block where height = {}",
        block_height + 1,
    ))
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    assert!(pre_check.is_none());

    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let first_check = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test_index1.block where height = {}",
        block_height + 1,
    ))
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    assert!(first_check.is_none());

    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let final_check = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test_index1.block where height = {}",
        block_height + 2,
    ))
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    assert!(final_check.is_some());

    let row = final_check.unwrap();

    let id: i64 = row.get(0);
    let height: i64 = row.get(1);
    let timestamp: i64 = row.get(2);

    assert_eq!(height, (block_height + 2) as i64);
    assert!(timestamp > 0);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres", feature = "pg-embed-skip"))]
async fn test_can_trigger_and_index_tuple_events_postgres() {
    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());

    let pool = postgres_connection_pool().await;
    let mut srv = indexer_service_postgres().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srv.register_index_from_manifest(manifest)
        .await
        .expect("Failed to initialize indexer.");

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/tuples").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    fuel_node_handle.abort();

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.tupleentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let _id: i64 = row.get(0);
    let complex_a: &str = row.get(1);
    let complex_b: i64 = row.get(2);
    let simple_a: &str = row.get(3);

    assert_eq!(complex_a, "abcde");
    assert_eq!(complex_b, 54321);
    assert_eq!(simple_a, "hello world!");
}

#[actix_web::test]
async fn test_can_trigger_and_index_events_with_multiple_fuel_indexes() {
    use fuel_indexer_tests::fixtures::deploy_multiple_contracts;
    use fuels::{
        prelude::Provider,
        signers::fuel_crypto::coins_bip32::ecdsa::digest::typenum::Prod,
    };

    let fuel_node_handle = tokio::spawn(setup_example_test_fuel_node());
    dbg!("setup fuel node");
    let pool = postgres_connection_pool().await;
    let mut srvc = indexer_service_postgres().await;

    for i in 0..3 {
        let mut manifest: Manifest =
            serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST)
                .expect("Bad Yaml File");

        manifest.namespace = format!("fuel_indexer_test_index{}", i + 1);
        update_test_manifest_asset_paths(&mut manifest);

        srvc.register_index_from_manifest(manifest)
            .await
            .expect("Failed to initialize indexer.");
    }

    dbg!("registered indexes, deploying contracts");

    let wallet_path = Path::new(WORKSPACE_ROOT)
        .join("assets")
        .join("test-chain-config.json");
    let wallet_path_str = wallet_path.as_os_str().to_str().unwrap();
    let mut wallet =
        WalletUnlocked::load_keystore(wallet_path_str, defaults::WALLET_PASSWORD, None)
            .unwrap();

    let host = defaults::FUEL_NODE_ADDR.to_string();
    let provider = Provider::connect(&host).await.unwrap();
    wallet.set_provider(provider);

    dbg!("wallet: {:?}", &wallet);

    let contract_bin_path = Path::new(WORKSPACE_ROOT)
        .join("contracts")
        .join("fuel-indexer-test")
        .join("out")
        .join("debug")
        .join("fuel-indexer-test.bin");

    let count = 3;
    let contract_ids = deploy_multiple_contracts(&wallet, contract_bin_path, count)
        .await
        .unwrap();
    let contract_ids_str: Vec<String> =
        contract_ids.iter().map(|x| x.to_string()).collect();

    dbg!("contract_ids: {:?}", contract_ids_str);

    // let contract = connect_to_deployed_contract().await.unwrap();
    // let app = test::init_service(app(contract)).await;
    // let req = test::TestRequest::post().uri("/indicies").to_request();
    // let _ = app.call(req).await;

    // sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    // fuel_node_handle.abort();

    // for i in 0..3 {
    //     let mut conn = pool.acquire().await.unwrap();
    //     let table_name = format!("fuel_indexer_test_index{}_index1", i + 1);
    //     let statement = format!(
    //         "SELECT * FROM {}.block ORDER by height DESC LIMIT 1",
    //         table_name
    //     );
    //     let block_row = sqlx::query(&statement).fetch_one(&mut conn).await.unwrap();
    //     let block_height: i64 = block_row.get(0);
    //     assert!(block_height >= 1);
    // }
}
