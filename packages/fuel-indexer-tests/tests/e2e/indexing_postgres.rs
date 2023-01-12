use actix_service::Service;
use actix_web::test;
use fuel_indexer::IndexerService;
use fuel_indexer_database::{queries, IndexerConnection};
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{
    assets, defaults,
    fixtures::{
        connect_to_deployed_contract, indexer_service_postgres, postgres_connection,
        postgres_connection_pool, test_web::app,
    },
    utils::update_test_manifest_asset_paths,
};
use fuel_indexer_types::{Address, ContractId, Identity};
use hex::FromHex;
use lazy_static::lazy_static;
use sqlx::{
    pool::{Pool, PoolConnection},
    Postgres, Row,
};
use tokio::time::{sleep, Duration};

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_events_with_multiple_args_in_index_handler_postgres()
{
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

    let mut conn = pool.acquire().await.unwrap();
    let block_row =
        sqlx::query("SELECT * FROM fuel_indexer_test.block ORDER BY height DESC LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let height: i64 = block_row.get(1);
    let timestamp: i64 = block_row.get(2);
    assert!(height >= 1);
    assert!(timestamp > 0);

    let ping_row =
        sqlx::query("SELECT * FROM fuel_indexer_test.pingentity WHERE id = 12345")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let ping_value: i64 = ping_row.get(1);
    assert_eq!(ping_value, 12345);

    let pong_row =
        sqlx::query("SELECT * FROM fuel_indexer_test.pongentity WHERE id = 45678")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let pong_value: i64 = pong_row.get(1);
    assert_eq!(pong_value, 45678);

    let pung_row =
        sqlx::query("SELECT * FROM fuel_indexer_test.pungentity WHERE id = 123")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let pung_from: String = pung_row.get(3);
    let from_buff = <[u8; 33]>::from_hex(&pung_from).unwrap();

    let contract_buff = <[u8; 32]>::from_hex(
        "322ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(
        Identity::from(from_buff),
        Identity::ContractId(ContractId::from(contract_buff)),
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_callreturn_postgres() {
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

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.pungentity WHERE id = 3")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let value: i64 = row.get(1);
    let is_pung: bool = row.get(2);
    let pung_from: String = row.get(3);
    let from_buff = <[u8; 33]>::from_hex(&pung_from).unwrap();

    let addr_buff = <[u8; 32]>::from_hex(
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(value, 12345);
    assert!(is_pung);
    assert_eq!(
        Identity::from(from_buff),
        Identity::Address(Address::from(addr_buff)),
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_blocks_and_transactions_postgres() {
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

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.block WHERE height = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let id: String = row.get(0);
    let height: i64 = row.get(1);
    let timestamp: i64 = row.get(2);

    assert_eq!(height, 1);
    assert!(timestamp > 0);

    let row = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test.tx WHERE block = '{}'",
        id
    ))
    .fetch_all(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.len(), 2);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_ping_event_postgres() {
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

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.pingentity WHERE id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let id: i64 = row.get(0);
    let value: i64 = row.get(1);

    assert_eq!(id, 1);
    assert_eq!(value, 123);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_transfer_event_postgres() {
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

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.transfer LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let amount: i64 = row.get(3);
    let asset_id: &str = row.get(4);

    assert_eq!(amount, 1); // value is defined in test contract
    assert_eq!(asset_id, defaults::TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_log_event_postgres() {
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

    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test.log WHERE ra = 8675309 LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let ra: i64 = row.get(2);

    assert_eq!(ra, 8675309);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_logdata_event_postgres() {
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

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.pungentity WHERE id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let value: i64 = row.get(1);
    let is_pung: bool = row.get(2);
    let pung_from: String = row.get(3);
    let from_buff = <[u8; 33]>::from_hex(&pung_from).unwrap();

    let addr_buff = <[u8; 32]>::from_hex(
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(value, 456);
    assert!(is_pung);
    assert_eq!(
        Identity::from(from_buff),
        Identity::Address(Address::from(addr_buff)),
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_scriptresult_event_postgres() {
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
    let mut conn = pool.acquire().await.unwrap();

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.scriptresult LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let result: i64 = row.get(1);
    let gas_used: i64 = row.get(2);
    let data: String = row.get(3);

    let expected = hex::decode(&data)
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
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_transferout_event_postgres() {
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

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.transferout LIMIT 1")
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
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_messageout_event_postgres() {
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

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.messageout LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let message_id: &str = row.get(0);
    let recipient: &str = row.get(2);
    let amount: i64 = row.get(3);
    let len: i64 = row.get(5);

    // Message ID is different on each receipt, so we'll just check that it's well-formed
    assert_eq!(message_id.len(), 64);
    assert_eq!(
        recipient,
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(amount, 100);
    assert_eq!(len, 24);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_index_metadata_is_saved_when_indexer_macro_is_called_postgres() {
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

    let mut conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test.indexmetadataentity LIMIT 1")
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
    let pool = postgres_connection_pool().await;
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

    let mut conn = pool.acquire().await.unwrap();
    let pre_check = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test.block where height = {}",
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
        "SELECT * FROM fuel_indexer_test.block where height = {}",
        block_height + 1,
    ))
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    assert!(first_check.is_none());

    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let final_check = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test.block where height = {}",
        block_height + 2,
    ))
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    assert!(final_check.is_some());

    let row = final_check.unwrap();

    let id: String = row.get(0);
    let height: i64 = row.get(1);
    let timestamp: i64 = row.get(2);

    assert_eq!(height, (block_height + 2) as i64);
    assert!(timestamp > 0);
}
