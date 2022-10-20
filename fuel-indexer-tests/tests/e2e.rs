#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use fuel_indexer_tests::{
    defaults,
    fixtures::{http_client, postgres_connection},
};
use more_asserts as ma;
use sqlx::Row;
use tokio::time::{sleep, Duration};

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_blocks_and_transactions() {
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let mut conn = pool.acquire().await.unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/block")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query(
        "SELECT * FROM fuel_indexer_test.blockentity ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&mut conn)
    .await
    .unwrap();

    let id: i64 = row.get(0);
    let height: i64 = row.get(1);
    let producer: String = row.get(2);
    let _hash: String = row.get(3);
    let timestamp: i64 = row.get(4);

    assert_eq!(height, id);
    assert_eq!(
        producer,
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    );
    ma::assert_gt!(timestamp, 0);

    let row = sqlx::query(&format!(
        "SELECT * FROM fuel_indexer_test.transactionentity where block = {}",
        id
    ))
    .fetch_all(&mut conn)
    .await
    .unwrap();

    assert_eq!(row.len(), 1);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_ping_event() {
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let mut conn = pool.acquire().await.unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/ping")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.pingentity where id = 1")
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
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let _conn = pool.acquire().await.unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/transfer")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    // FIXME: Still need to trigger an actual receipt
    assert_eq!(1, 1);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_log_event() {
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let mut conn = pool.acquire().await.unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/log")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.logentity where id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let _id: i64 = row.get(0);
    let _contract_id: String = row.get(1);
    let ra: i64 = row.get(2);
    let _rb: i64 = row.get(3);

    assert_eq!(ra, defaults::PONG_EVENT_VALUE); // value is defined in test contract
}

// #[tokio::test]
// #[cfg(feature = "e2e")]
// async fn test_can_trigger_and_index_logdata_event() {
//     let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
//     let mut conn = pool.acquire().await.unwrap();

//     let client = http_client();
//     let _ = client
//         .post("http://127.0.0.1:8000/logdata")
//         .send()
//         .await
//         .unwrap();

//     sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

//     // TODO: finish
//     assert_eq!(1, 1);
// }

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_scriptresult_event() {
    use ma::assert_gt;

    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let mut conn = pool.acquire().await.unwrap();

    // Remove any lingering ScriptResult items
    sqlx::query("DELETE FROM fuel_indexer_test.scriptresultentity WHERE id IS NOT NULL")
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

    let row =
        sqlx::query("SELECT * FROM fuel_indexer_test.scriptresultentity where id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

    let _id: i64 = row.get(0);
    let result: i64 = row.get(1);
    let gas_used: i64 = row.get(2);
    assert_eq!(result, 0);
    assert_gt!(gas_used, 0);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_transferout_event() {
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let _conn = pool.acquire().await.unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/transferout")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    // FIXME: Still need to trigger an actual receipt
    assert_eq!(1, 1);
}

// TODO: Revisit after https://github.com/FuelLabs/sway/issues/2899
// merges as it adds support for send_message_with_output()
#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_messageout_event() {
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let _conn = pool.acquire().await.unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/messageout")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(defaults::INDEXED_EVENT_WAIT)).await;

    // FIXME: Still need to trigger an actual receipt
    assert_eq!(1, 1);
}
