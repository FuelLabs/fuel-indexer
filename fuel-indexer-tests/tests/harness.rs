use fuel_indexer_tests::fixtures::{http_client, postgres_connection};
use sqlx;
use sqlx::Row;

#[tokio::test]
async fn test_can_trigger_and_index_ping_event() {
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

    let id: i64 = row.get(0);
    let ping: i64 = row.get(1);
    let pong: i64 = row.get(2);
    let _message: String = row.get(3);

    assert_eq!(id, 1);
    assert_eq!(ping, 123);
    assert_eq!(pong, 456);
}
