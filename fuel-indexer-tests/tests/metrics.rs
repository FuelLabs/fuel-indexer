#![cfg_attr(not(feature = "e2e"), allow(dead_code, unused_imports))]
use fuel_indexer_lib::config::IndexerConfig;
use fuel_indexer_postgres as postgres;
use fuel_indexer_sqlite as sqlite;
use fuel_indexer_tests::fixtures::{
    http_client, postgres_connection, sqlite_connection_pool,
};

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_metrics_endpoint_returns_proper_count_of_metrics() {
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

    assert_eq!(resp.split('\n').count(), 184);
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_database_postgres_metrics_properly_increments_counts_when_queries_are_made()
{
    let mut conn = postgres_connection().await;

    let _ = postgres::start_transaction(&mut conn);

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

    assert_eq!(
        categories[81],
        "# HELP postgres_start_transaction_calls Count of calls to postgres start_transaction_calls."
    );
    assert_eq!(
        categories[82],
        "# TYPE postgres_start_transaction_calls counter"
    );
    assert!(
        categories[83].split(' ').collect::<Vec<&str>>()[1]
            .to_string()
            .parse::<i64>()
            .unwrap()
            >= 1,
    );
}

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_database_sqlite_metrics_properly_increments_counts_when_queries_are_made() {
    let mut conn = sqlite_connection_pool().await;

    let _ = sqlite::start_transaction(&mut conn);

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

    assert_eq!(
        categories[168],
        "# HELP sqlite_start_transaction_calls Count of calls to sqlite start_transaction_calls."
    );
    assert_eq!(
        categories[169],
        "# TYPE sqlite_start_transaction_calls counter"
    );
    assert!(
        categories[170].split(' ').collect::<Vec<&str>>()[1]
            .to_string()
            .parse::<i64>()
            .unwrap()
            >= 1,
    );
}
