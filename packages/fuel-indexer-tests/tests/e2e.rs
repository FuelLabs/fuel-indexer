#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_tests::{
    assets, defaults,
    fixtures::{http_client, indexer_service, postgres_connection},
    utils::update_test_manifest_asset_paths,
};
use fuel_indexer_types::{Address, Identity};
use hex::FromHex;
use sqlx::Row;
use tokio::time::{sleep, Duration};

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_callreturn() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/callreturn")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

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

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_blocks_and_transactions() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/block")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

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

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_ping_event() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/ping")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.pingentity WHERE id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let id: i64 = row.get(0);
    let value: i64 = row.get(1);

    assert_eq!(id, 1);
    assert_eq!(value, 123);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_transfer_event() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    sqlx::query("DELETE FROM fuel_indexer_test.transfer WHERE id IS NOT NULL")
        .execute(&mut conn)
        .await
        .unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/transfer")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.transfer LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let amount: i64 = row.get(3);
    let asset_id: &str = row.get(4);

    assert_eq!(amount, 1); // value is defined in test contract
    assert_eq!(asset_id, defaults::TRANSFER_BASE_ASSET_ID);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_log_event() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    sqlx::query("DELETE FROM fuel_indexer_test.log WHERE id IS NOT NULL")
        .execute(&mut conn)
        .await
        .unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/log")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.log LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let ra: i64 = row.get(2);

    assert_eq!(ra, 8675309);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_logdata_event() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/logdata")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

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

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_scriptresult_event() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    sqlx::query("DELETE FROM fuel_indexer_test.scriptresult WHERE id IS NOT NULL")
        .execute(&mut conn)
        .await
        .unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/scriptresult")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.scriptresult LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let result: i64 = row.get(1);
    let gas_used: i64 = row.get(2);

    assert!((0..=1).contains(&result));
    assert!(gas_used > 0);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_transferout_event() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/transferout")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

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

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_messageout_event() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/messageout")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

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

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_index_metadata_is_saved_when_indexer_macro_is_called() {
    let mut conn = postgres_connection().await;
    let mut srvc = indexer_service().await;
    let mut manifest: Manifest =
        serde_yaml::from_str(assets::FUEL_INDEXER_TEST_MANIFEST).expect("Bad yaml file.");

    update_test_manifest_asset_paths(&mut manifest);

    srvc.register_indices(Some(manifest))
        .await
        .expect("Failed to initialize indexer.");

    srvc.run().await;

    sqlx::query("DELETE FROM fuel_indexer_test.indexmetadataentity WHERE id IS NOT NULL")
        .execute(&mut conn)
        .await
        .unwrap();

    let client = http_client();
    // Doesn't matter what event we trigger
    let _ = client
        .post("http://127.0.0.1:8000/ping")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.indexmetadataentity LIMIT 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let block_height: i64 = row.get(0);
    let time: i64 = row.get(1);

    assert!(block_height >= 1);
    assert!(time >= 1);
}
