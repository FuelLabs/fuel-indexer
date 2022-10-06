#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use fuel_indexer_tests::fixtures::{http_client, postgres_connection};
use sqlx::Row;
use tokio::time::{sleep, Duration};

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

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.message where id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    let id: i64 = row.get(0);
    let ping: i64 = row.get(1);
    let pong: i64 = row.get(2);
    let _message: String = row.get(3);

    assert_eq!(id, 1);
    assert_eq!(ping, 123);
    assert_eq!(pong, 456);
}
