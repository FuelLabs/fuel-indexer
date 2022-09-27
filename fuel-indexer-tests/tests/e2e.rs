use fuel_indexer::Manifest;
use fuel_indexer_tests::fixtures::{http_client, postgres_connection};
use sqlx;
use sqlx::Row;
use std::path::Path;

const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[tokio::test]
async fn test_can_trigger_ping_event() {
    let pool = postgres_connection("postgres://postgres@127.0.0.1").await;
    let mut conn = pool.acquire().await.unwrap();

    let client = http_client();
    let response: u64 = client
        .post("http://0.0.0.0:8000/ping")
        .send()
        .await
        .unwrap()
        .json::<u64>()
        .await
        .unwrap();

    assert_eq!(response, 123);

    let row = sqlx::query("SELECT * FROM fuel_indexer_test.message where id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.len(), 1);
}
