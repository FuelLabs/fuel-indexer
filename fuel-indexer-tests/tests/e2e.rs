#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use fuel_indexer_tests::fixtures::{
    postgres_connection, setup_contract, setup_indexer_service,
    setup_test_client_and_wallet, tx_params, Ping,
};
use sqlx::Row;
use tokio::time::{sleep, Duration};

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_can_trigger_and_index_ping_event() {
    let pool = postgres_connection("postgres://postgres:my-secret@127.0.0.1").await;
    let mut conn = pool.acquire().await.unwrap();

    let wallet = setup_test_client_and_wallet().await;
    let contract = setup_contract(wallet).await;

    setup_indexer_service().await;

    let result = contract.ping().tx_params(tx_params()).call().await.unwrap();
    let ping: Ping = result.value;
    assert_eq!(ping.value.to_string(), "123".to_string());

    // Events are not triggered immediately
    sleep(Duration::from_millis(3000)).await;

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
