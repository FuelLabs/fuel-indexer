use actix_service::Service;
use actix_web::test;
use bigdecimal::ToPrimitive;
use fuel_indexer::IndexerService;
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{
    assets, defaults,
    fixtures::{
        connect_to_deployed_contract, indexer_service_postgres,
        setup_example_test_fuel_node, test_web::app, TestPostgresDb,
    },
    utils::update_test_manifest_asset_paths,
};
use fuel_indexer_types::{Address, ContractId, Identity};
use sqlx::{types::BigDecimal, Row};
use std::str::FromStr;
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};

const REVERT_VM_CODE: u64 = 0x0004;
const EXPECTED_CONTRACT_ID: &str =
    "e9fcc111f5273447709689198d2059eb490b666e0b1f6e272ef7f960c685f2a5";

async fn setup_test_components(
) -> (JoinHandle<Result<(), ()>>, TestPostgresDb, IndexerService) {
    let node_handle = tokio::spawn(setup_example_test_fuel_node());
    let test_db = TestPostgresDb::new().await.unwrap();
    let srvc = indexer_service_postgres(Some(&test_db.url)).await;

    (node_handle, test_db, srvc)
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_events_with_multiple_args_in_index_handler_postgres()
{
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/multiarg").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let block_row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.block ORDER BY height DESC LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let height = block_row.get::<BigDecimal, usize>(1).to_u64().unwrap();
    let timestamp: i64 = block_row.get(2);
    assert!(height >= 1);
    assert!(timestamp > 0);

    let ping_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pingentity WHERE id = 12345")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let ping_value = ping_row.get::<BigDecimal, usize>(1).to_u64().unwrap();
    assert_eq!(ping_value, 12345);

    let pong_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pongentity WHERE id = 45678")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let pong_value = pong_row.get::<BigDecimal, usize>(1).to_u64().unwrap();
    assert_eq!(pong_value, 45678);

    let pung_row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 123")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let from_buff = ContractId::from_str(&pung_row.get::<String, usize>(3)).unwrap();

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
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/callreturn").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 3")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let from_buff = Address::from_str(&row.get::<String, usize>(3)).unwrap();

    let addr_buff = Address::from_str(
        "0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(12345, row.get::<BigDecimal, usize>(1).to_u64().unwrap());
    assert!(row.get::<bool, usize>(2));
    assert_eq!(Identity::Address(from_buff), Identity::Address(addr_buff));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_blocks_and_transactions_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::get().uri("/block_height").to_request();
    let res = test::call_and_read_body(&app, req).await;
    let block_height = String::from_utf8(res.to_vec())
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let req = test::TestRequest::post().uri("/block").to_request();
    let _ = app.call(req).await;
    node_handle.abort();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.block ORDER BY timestamp DESC LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let id = row.get::<BigDecimal, usize>(0).to_u64().unwrap();
    let timestamp = row.get::<i64, usize>(2);

    assert_eq!(
        row.get::<BigDecimal, usize>(1).to_u64().unwrap(),
        block_height + 1
    );
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
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pingentity WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1);
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 123);

    // Ping also triggers the 128-bit integer test as well
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.u16entity WHERE id = 9999")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert_eq!(
        row.get::<BigDecimal, usize>(1),
        BigDecimal::from_str("340282366920938463463374607431768211454").unwrap()
    );
    assert_eq!(
        row.get::<BigDecimal, usize>(2),
        BigDecimal::from_str("170141183460469231731687303715884105727").unwrap()
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_transfer_event_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/transfer").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.transfer LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 1); // value is defined in test contract
    assert_eq!(row.get::<&str, usize>(4), defaults::TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_log_event_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/log").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.log WHERE ra = 8675309 LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(2).to_u64().unwrap(), 8675309);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_logdata_event_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/logdata").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.pungentity WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let from_buff = Address::from_str(&row.get::<String, usize>(3)).unwrap();
    let addr_buff = Address::from_str(
        "0x532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96",
    )
    .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 456);
    assert!(row.get::<bool, usize>(2));
    assert_eq!(Identity::Address(from_buff), Identity::Address(addr_buff));
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_scriptresult_event_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/scriptresult").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.scriptresult LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let expected = hex::decode(row.get::<String, usize>(3))
        .unwrap()
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");

    assert!((0..=1).contains(&row.get::<BigDecimal, usize>(1).to_u64().unwrap()));
    assert!(row.get::<BigDecimal, usize>(2).to_u64().unwrap() > 0);
    assert_eq!(expected, "1,1,1,1,1".to_string());
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_transferout_event_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/transferout").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.transferout LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(
        row.get::<&str, usize>(2),
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(row.get::<BigDecimal, usize>(3).to_u64().unwrap(), 1);
    assert_eq!(row.get::<&str, usize>(4), defaults::TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_messageout_event_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/messageout").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.messageout LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let recipient = row.get::<&str, usize>(3);
    let amount = row.get::<BigDecimal, usize>(4).to_u64().unwrap();
    assert_eq!(
        recipient,
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(amount, 100);

    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.messageentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 1234);
    assert_eq!(
        row.get::<&str, usize>(1),
        "abcdefghijklmnopqrstuvwxyz123456"
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_index_event_with_optional_fields_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/optionals").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test_index1.optionentity WHERE id = 8675309",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let opt_int = row.get::<Option<BigDecimal>, usize>(2);

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 8675309);
    assert_eq!(row.get::<BigDecimal, usize>(1).to_u64().unwrap(), 100);
    assert!(opt_int.is_some());

    assert_eq!(opt_int.unwrap().to_u64().unwrap(), 999);

    assert!(row.get::<Option<&str>, usize>(3).is_none());
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_index_metadata_is_saved_when_indexer_macro_is_called_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test_index1.indexmetadataentity LIMIT 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    assert!(row.get::<BigDecimal, usize>(0).to_u64().unwrap() >= 1);
    assert!(row.get::<i64, usize>(1) >= 1);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_index_respects_start_block_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::get().uri("/block_height").to_request();
    let res = test::call_and_read_body(&app, req).await;
    let block_height = String::from_utf8(res.to_vec())
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    manifest.start_block = Some(block_height + 2);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let mut conn = test_db.pool.acquire().await.unwrap();
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
    node_handle.abort();

    let final_check = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test_index1.block where height = {}",
        block_height + 2,
    ))
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    assert!(final_check.is_some());

    let row = final_check.unwrap();
    let height = row.get::<BigDecimal, usize>(1).to_u64().unwrap();

    assert_eq!(height, (block_height + 2));
    assert!(row.get::<i64, usize>(2) > 0);
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_tuple_events_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/tuples").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.tupleentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<&str, usize>(1), "abcde");
    assert_eq!(row.get::<BigDecimal, usize>(2).to_u64().unwrap(), 54321);
    assert_eq!(row.get::<&str, usize>(3), "hello world!");
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_pure_function_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/pure_function").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.callentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 123);
    assert_eq!(
        row.get::<&str, usize>(1),
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert_eq!(row.get::<&str, usize>(3), defaults::TRANSFER_BASE_ASSET_ID);
    assert_eq!(row.get::<&str, usize>(5), "trigger_pure_function");
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_revert_function_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/revert").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.revertentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 123);
    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(
        row.get::<BigDecimal, usize>(2).to_u64().unwrap(),
        REVERT_VM_CODE
    );
}

#[actix_web::test]
#[cfg(all(feature = "e2e", feature = "postgres"))]
async fn test_can_trigger_and_index_panic_function_postgres() {
    let (node_handle, test_db, mut srvc) = setup_test_components().await;

    let mut manifest = Manifest::try_from(assets::FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_index_from_manifest(manifest).await.unwrap();

    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let req = test::TestRequest::post().uri("/panic").to_request();
    let _ = app.call(req).await;

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;
    node_handle.abort();

    let mut conn = test_db.pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM fuel_indexer_test_index1.panicentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.get::<BigDecimal, usize>(0).to_u64().unwrap(), 123);
    assert_eq!(row.get::<&str, usize>(1), EXPECTED_CONTRACT_ID);
    assert_eq!(row.get::<i32, usize>(2), 5);
}
