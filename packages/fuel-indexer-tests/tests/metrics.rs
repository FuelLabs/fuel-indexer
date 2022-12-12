use fuel_indexer_lib::config::IndexerConfig;
use fuel_indexer_postgres as postgres;
use fuel_indexer_sqlite as sqlite;
use fuel_indexer_tests::fixtures::{
    http_client, indexer_service, postgres_connection, sqlite_connection,
};

#[tokio::test]
#[cfg(feature = "e2e")]
async fn test_metrics_endpoint_returns_proper_count_of_metrics() {
    let client = http_client();
    let resp = client
        .get("http://127.0.0.1:29987/api/metrics")
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
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into());
    let _ = postgres::execute_query(&mut conn, "SELECT 1;".into());

    let client = http_client();
    let config = IndexerConfig::default();

    let resp = client
        .get("http://127.0.0.1:29987/api/metrics")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let categories = resp.split('\n').collect::<Vec<&str>>();

    assert_eq!(
        categories[18],
        "# HELP postgres_execute_query_calls Count of calls to postgres execute_query_calls."
    );
    assert_eq!(
        categories[19],
        "# TYPE postgres_execute_query_calls counter"
    );

    assert!(
        categories[20].split(' ').collect::<Vec<&str>>()[1]
            .to_string()
            .parse::<i64>()
            .unwrap()
            >= 1,
    );
}

// The following test works, but requires a SQLite DB connection
//
// We can uncomment this out after https://github.com/FuelLabs/fuel-indexer/issues/269
// when we can spin up service on-demand with arbitrary database connections
// #[tokio::test]
// #[cfg(feature = "e2e")]
// async fn test_database_sqlite_metrics_properly_increments_counts_when_queries_are_made() {
//     let mut conn = sqlite_connection().await;

//     let _ = sqlite::start_transaction(&mut conn);

//     let client = http_client();

//     let config = IndexerConfig::default();

//     let resp = client
//         .get(format!(
//             "http://{}:{}/api/metrics",
//             config.graphql_api.host, config.graphql_api.port
//         ))
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();

//     let categories = resp.split('\n').collect::<Vec<&str>>();

//     assert_eq!(
//         categories[168],
//         "# HELP sqlite_start_transaction_calls Count of calls to sqlite start_transaction_calls."
//     );
//     assert_eq!(
//         categories[169],
//         "# TYPE sqlite_start_transaction_calls counter"
//     );
//     assert!(
//         categories[170].split(' ').collect::<Vec<&str>>()[1]
//             .to_string()
//             .parse::<i64>()
//             .unwrap()
//             >= 1,
//     );
// }
