#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_lib::config::IndexerConfig;
use fuel_indexer_tests::fixtures::http_client;

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_database_metrics_properly_increments_counts_when_queries_are_made() {
    let pool = IndexerConnectionPool::connect("postgres://postgres:my-secret@127.0.0.1")
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let _ = queries::start_transaction(&mut conn);

    let client = http_client();

    let config = IndexerConfig::default();

    let resp = client
        .get(format!(
            "http://{}:{}/api/metrics",
            config.graphql_api.host, config.graphql_api.port
        ))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let categories = resp.split('\n').collect::<Vec<&str>>();

    assert_eq!(categories.len(), 100);

    assert_eq!(
        categories[84],
        "# HELP start_transaction Count of calls to start_transaction."
    );
    assert_eq!(categories[85], "# TYPE start_transaction counter");
    assert!(
        categories[86].split(' ').collect::<Vec<&str>>()[1]
            .to_string()
            .parse::<i64>()
            .unwrap()
            >= 1,
    );
}
