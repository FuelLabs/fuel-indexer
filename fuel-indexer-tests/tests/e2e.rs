#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use fuel_indexer_tests::fixtures::{http_client, postgres_connection};
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

    // Events are not triggered immediately
    sleep(Duration::from_secs(4)).await;

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

    // Events are not triggered immediately
    sleep(Duration::from_secs(4)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.pingentity where id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let id: i64 = row.get(0);
    let value: i64 = row.get(1);

    assert_eq!(id, 1);
    assert_eq!(value, 123);
}

// #[tokio::test]
// #[cfg(feature = "e2e")]
// async fn test_can_trigger_and_index_transfer_event() {
//     let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
//     let mut conn = pool.acquire().await.unwrap();

//     let client = http_client();
//     let _ = client
//         .post("http://127.0.0.1:8000/transfer")
//         .send()
//         .await
//         .unwrap();

//     // Events are not triggered immediately
//     sleep(Duration::from_secs(4)).await;

//     // TODO: finish
//     assert_eq!(1, 1);
// }

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

    // Events are not triggered immediately
    sleep(Duration::from_secs(4)).await;

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.logentity where id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let _id: i64 = row.get(0);
    let _contract_id: String = row.get(1);
    let ra: i64 = row.get(2);
    let _rb: i64 = row.get(3);

    assert_eq!(ra, 8675309); // value is defined in test contract
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

//     // Events are not triggered immediately
//     sleep(Duration::from_secs(4)).await;

//     // TODO: finish
//     assert_eq!(1, 1);
// }

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_scriptresult_event() {
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let mut conn = pool.acquire().await.unwrap();

    let client = http_client();
    let _ = client
        .post("http://127.0.0.1:8000/scriptresult")
        .send()
        .await
        .unwrap();

    sleep(Duration::from_secs(4)).await;

    let row = sqlx::query("SELECT * from fuel_indexer_test.scriptresult where id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let result = row.get(0);
    let gas_used = row.get(1);

    // Values are defined in the test script
    assert_eq!(result, 0x00);
    assert_eq!(gas_used, 100);
}
