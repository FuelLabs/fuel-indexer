#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use actix_web::test;
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_postgres as postgres;
use fuel_indexer_tests::{
    assets::FUEL_INDEXER_TEST_MANIFEST,
    defaults,
    fixtures::{
        connect_to_deployed_contract, indexer_service, postgres_connection,
        setup_test_client_and_deploy_contract, test_web::app,
    },
    utils::update_test_manifest_asset_paths,
};
use sqlx::Row;

#[actix_web::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_blocks_and_transactions() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;
    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;

    // Cleanup
    let _ = postgres::start_transaction(&mut conn).await;
    sqlx::query("DELETE FROM fuel_indexer_test.tx WHERE id IS NOT NULL")
        .execute(&mut conn)
        .await
        .unwrap();
    sqlx::query("DELETE FROM fuel_indexer_test.block WHERE height IS NOT NULL")
        .execute(&mut conn)
        .await
        .unwrap();
    let _ = postgres::commit_transaction(&mut conn).await;

    srv.run().await;

    let req = test::TestRequest::post().uri("/block").to_request();
    let _ = test::call_service(&app, req).await;

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
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_ping_event() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;
    srv.run().await;

    let req = test::TestRequest::post().uri("/ping").to_request();
    let _ = test::call_service(&app, req).await;

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
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_transfer_event() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;
    srv.run().await;

    let req = test::TestRequest::post().uri("/transfer").to_request();
    let _ = test::call_service(&app, req).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.transfer LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let amount: i64 = row.get(3);
    let asset_id: &str = row.get(4);

    assert_eq!(amount, 1);
    assert_eq!(asset_id, defaults::TRANSFER_BASE_ASSET_ID);
}

#[actix_web::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_log_event() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;
    srv.run().await;

    let req = test::TestRequest::post().uri("/log").to_request();
    let _ = test::call_service(&app, req).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.log LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let ra: i64 = row.get(2);

    assert_eq!(ra, 8675309);
}

#[actix_web::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_logdata_event() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;
    srv.run().await;

    let req = test::TestRequest::post().uri("/logdata").to_request();
    let _ = test::call_service(&app, req).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.pungentity WHERE id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let value: i64 = row.get(1);
    let is_pung: i32 = row.get(2);

    assert_eq!(value, 456);
    assert_eq!(is_pung, 1);
}

#[actix_web::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_scriptresult_event() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;

    // Cleanup
    let _ = postgres::start_transaction(&mut conn).await;
    sqlx::query("DELETE FROM fuel_indexer_test.scriptresult WHERE id IS NOT NULL")
        .execute(&mut conn)
        .await
        .unwrap();
    let _ = postgres::commit_transaction(&mut conn).await;

    srv.run().await;

    let req = test::TestRequest::post().uri("/scriptresult").to_request();
    let _ = test::call_service(&app, req).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.scriptresult LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let result: i64 = row.get(1);
    let gas_used: i64 = row.get(2);

    assert!((0..=1).contains(&result));
    assert!(gas_used > 0);
}

#[actix_web::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_transferout_event() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let _conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;

    srv.run().await;

    let req = test::TestRequest::post().uri("/transferout").to_request();
    let _ = test::call_service(&app, req).await;

    // FIXME: Still need to trigger an actual receipt
    assert_eq!(1, 1);
}

#[actix_web::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_messageout_event() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;
    srv.run().await;

    let req = test::TestRequest::post().uri("/messageout").to_request();
    let _ = test::call_service(&app, req).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.messageout LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let recipient: &str = row.get(2);
    let amount: i64 = row.get(3);
    let len: i64 = row.get(5);

    assert_eq!(
        recipient,
        "532ee5fb2cabec472409eb5f9b42b59644edb7bf9943eda9c2e3947305ed5e96"
    );
    assert_eq!(amount, 100);
    assert_eq!(len, 24);
}

#[actix_web::test]
#[cfg(feature = "e2e")]
async fn test_index_metadata_is_saved_when_indexer_macro_is_called() {
    let contract = connect_to_deployed_contract().await.unwrap();
    let app = test::init_service(app(contract)).await;
    let mut conn = postgres_connection().await;

    let mut manifest = Manifest::from_str_contents(FUEL_INDEXER_TEST_MANIFEST).unwrap();
    update_test_manifest_asset_paths(&mut manifest);
    let srv = indexer_service(manifest, None).await;
    srv.run().await;

    let req = test::TestRequest::post().uri("/blocks").to_request();
    let _ = test::call_service(&app, req).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.indexmetadataentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let block_height: i64 = row.get(0);
    let time: i64 = row.get(1);

    assert!(block_height >= 1);
    assert!(time >= 1);
}
